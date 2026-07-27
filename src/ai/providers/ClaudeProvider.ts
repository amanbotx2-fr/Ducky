import {
  MAXIMUM_AI_MODEL_CANDIDATES,
  MAXIMUM_AI_OUTPUT_TOKENS,
} from '../AIAbuseLimits';
import {
  type AIOperationOptions,
  type AIConnectionResult,
  type AIModel,
  type AIProvider,
  type AIProviderConfiguration,
  type AIRequest,
  type AIResponse,
  type AIStreamChunk,
} from '../AIProvider';
import {
  createConfigurationError,
  createEmptyResponseError,
  createStreamingUnsupportedError,
  normalizeModels,
  toProviderError,
} from './providerUtils';

const ANTHROPIC_API_BASE_URL = 'https://api.anthropic.com/v1';
const ANTHROPIC_VERSION = '2023-06-01';
const REQUEST_TIMEOUT_MS = 30_000;
const MODEL_PAGE_SIZE = 1_000;

interface AnthropicContentBlock {
  readonly type?: string;
  readonly text?: string;
}

interface AnthropicUsage {
  readonly input_tokens?: number;
  readonly output_tokens?: number;
}

interface AnthropicMessageResponse {
  readonly content?: readonly AnthropicContentBlock[];
  readonly stop_reason?: string;
  readonly usage?: AnthropicUsage;
}

interface AnthropicModel {
  readonly id?: string;
  readonly display_name?: string;
}

interface AnthropicModelsResponse {
  readonly data?: readonly AnthropicModel[];
}

const parseJsonResponse = async <T>(
  response: Response,
): Promise<T> => {
  if (!response.ok) {
    const error = new Error(
      `Anthropic request failed with status ${response.status}.`,
    ) as Error & { status: number };
    error.status = response.status;
    throw error;
  }

  return (await response.json()) as T;
};

const withTimeoutSignal = (
  signal: AbortSignal | undefined,
): AbortSignal =>
  signal === undefined
    ? AbortSignal.timeout(REQUEST_TIMEOUT_MS)
    : AbortSignal.any([
        signal,
        AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      ]);

export class ClaudeProvider implements AIProvider {
  public readonly id = 'claude' as const;
  public readonly displayName = 'Claude';
  private apiKey = '';
  private model = '';

  public initialize(
    configuration: AIProviderConfiguration,
  ): Promise<void> {
    this.apiKey = configuration.apiKey.trim();
    this.model = configuration.model.trim();
    return Promise.resolve();
  }

  public isConfigured(): boolean {
    return this.apiKey.length > 0 && this.model.length > 0;
  }

  public async sendMessage(
    request: AIRequest,
    options: AIOperationOptions = {},
  ): Promise<AIResponse> {
    const apiKey = this.requireApiKey();
    const model = this.requireModel();

    try {
      const response = await fetch(
        `${ANTHROPIC_API_BASE_URL}/messages`,
        {
          method: 'POST',
          headers: this.createHeaders(apiKey),
          body: JSON.stringify({
            model,
            max_tokens: MAXIMUM_AI_OUTPUT_TOKENS,
            messages: [{ role: 'user', content: request.prompt }],
          }),
          signal: withTimeoutSignal(options.signal),
        },
      );
      const message =
        await parseJsonResponse<AnthropicMessageResponse>(response);
      const content = (message.content ?? [])
        .filter((block) => block.type === 'text')
        .map((block) => block.text ?? '')
        .join('')
        .trim();

      if (content.length === 0) {
        throw createEmptyResponseError(this.id, this.displayName);
      }

      return {
        providerId: this.id,
        content,
        finishReason:
          message.stop_reason === 'max_tokens' ? 'length' : 'stop',
        ...(message.usage === undefined
          ? {}
          : {
              usage: {
                inputTokens: message.usage.input_tokens ?? 0,
                outputTokens: message.usage.output_tokens ?? 0,
              },
            }),
      };
    } catch (error) {
      throw toProviderError(this.id, this.displayName, error);
    }
  }

  public async listModels(
    options: AIOperationOptions = {},
  ): Promise<readonly AIModel[]> {
    const apiKey = this.requireApiKey();

    try {
      const url = new URL(`${ANTHROPIC_API_BASE_URL}/models`);
      url.searchParams.set('limit', String(MODEL_PAGE_SIZE));
      const response = await fetch(url, {
        headers: this.createHeaders(apiKey),
        signal: withTimeoutSignal(options.signal),
      });
      const result =
        await parseJsonResponse<AnthropicModelsResponse>(response);
      const models: AIModel[] = [];

      for (const model of (
        result.data ?? []
      ).slice(0, MAXIMUM_AI_MODEL_CANDIDATES)) {
        if (typeof model.id !== 'string') {
          continue;
        }

        models.push({
          id: model.id,
          ...(typeof model.display_name === 'string'
            ? { displayName: model.display_name }
            : {}),
        });
      }

      return normalizeModels(models);
    } catch (error) {
      throw toProviderError(this.id, this.displayName, error);
    }
  }

  public async testConnection(
    options: AIOperationOptions = {},
  ): Promise<AIConnectionResult> {
    await this.listModels(options);
    return {
      message: 'Connection successful.',
    };
  }

  public async *streamMessage(
    _request: AIRequest,
  ): AsyncIterable<AIStreamChunk> {
    throw createStreamingUnsupportedError(this.id, this.displayName);
  }

  public dispose(): Promise<void> {
    this.apiKey = '';
    this.model = '';
    return Promise.resolve();
  }

  private createHeaders(apiKey: string): Readonly<Record<string, string>> {
    return {
      'anthropic-version': ANTHROPIC_VERSION,
      'content-type': 'application/json',
      'x-api-key': apiKey,
    };
  }

  private requireApiKey(): string {
    if (this.apiKey.length === 0) {
      throw createConfigurationError(
        this.id,
        'Claude requires an API key.',
      );
    }

    return this.apiKey;
  }

  private requireModel(): string {
    if (this.model.length === 0) {
      throw createConfigurationError(
        this.id,
        'Claude requires a model.',
      );
    }

    return this.model;
  }
}

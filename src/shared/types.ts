import type {
  AIModel,
  AIProviderHttpDiagnostics,
  AIResponse,
} from '../ai/AIProvider';
import type { AIConversationRequest } from './aiConversation';
import type {
  CredentialId,
  CredentialStatus,
} from './credentials';
import type { DailyPlannerBriefing } from './dailyPlanner';
import type {
  PomodoroCompletionListener,
  PomodoroCustomDurationRequestListener,
  PomodoroState,
  PomodoroStateListener,
} from './pomodoro';
import type {
  CreateReminderInput,
  Reminder,
  ReminderFiredNotification,
  UpdateReminderInput,
} from './reminders';
import type {
  AiConfigurationUpdate,
  PreferencesSettings,
  PreferencesSettingsPatch,
  RuntimeSettings,
} from './settings';
import type {
  UpdateStatus,
  UpdateStatusListener,
} from './updates';

export interface ScreenPoint {
  readonly x: number;
  readonly y: number;
}

export type CursorPositionListener = (position: ScreenPoint) => void;

export interface CompanionWindowBridge {
  readonly getCursorPosition: () => Promise<ScreenPoint>;
  readonly getWindowPosition: () => Promise<ScreenPoint>;
  readonly onCursorPosition: (
    listener: CursorPositionListener,
  ) => () => void;
  readonly moveWindow: (position: ScreenPoint) => void;
  readonly setCompanionContentHeight: (height: number) => void;
  readonly showCompanionContextMenu: () => void;
}

export type RuntimeSettingsChangeListener = (
  settings: RuntimeSettings,
) => void;
export type UserNamePanelRequestListener = () => void;
export type StickyMessagePanelRequestListener = () => void;
export type ReminderCreationPanelRequestListener = () => void;
export type ReminderManagerPanelRequestListener = () => void;
export type DailyPlannerPanelRequestListener = () => void;
export type ReminderFiredListener = (
  notification: ReminderFiredNotification,
) => void;

export type AIAskResult =
  | {
      readonly ok: true;
      readonly response: AIResponse;
    }
  | {
      readonly ok: false;
      readonly message: string;
    };

export type AIModelListResult =
  | {
      readonly ok: true;
      readonly models: readonly AIModel[];
    }
  | {
      readonly ok: false;
      readonly message: string;
    };

export type AIConnectionTestResult =
  | {
      readonly ok: true;
      readonly message: string;
    }
  | {
      readonly ok: false;
      readonly message: string;
      readonly diagnostics?: AIProviderHttpDiagnostics;
    };

export interface SettingsChangeBridge {
  readonly onRuntimeSettingsChanged: (
    listener: RuntimeSettingsChangeListener,
  ) => () => void;
}

export interface RuntimeSettingsBridge extends SettingsChangeBridge {
  readonly getRuntimeSettings: () => Promise<RuntimeSettings>;
}

export interface CompanionSettingsBridge extends RuntimeSettingsBridge {
  readonly updateUserName: (name: string) => Promise<string>;
  readonly updateStickyMessage: (
    message: string | null,
  ) => Promise<string | null>;
}

export interface PreferencesSettingsBridge extends SettingsChangeBridge {
  readonly getPreferencesSettings: () => Promise<PreferencesSettings>;
  readonly updatePreferencesSettings: (
    patch: PreferencesSettingsPatch,
  ) => Promise<PreferencesSettings>;
}

export interface CredentialBridge {
  readonly getCredentialStatus: (
    id: CredentialId,
  ) => Promise<CredentialStatus>;
  readonly saveCredential: (
    id: CredentialId,
    secret: string,
  ) => Promise<CredentialStatus>;
  readonly deleteCredential: (
    id: CredentialId,
  ) => Promise<CredentialStatus>;
}

export interface ReminderBridge {
  readonly onReminderCreationPanelRequested: (
    listener: ReminderCreationPanelRequestListener,
  ) => () => void;
  readonly onReminderManagerPanelRequested: (
    listener: ReminderManagerPanelRequestListener,
  ) => () => void;
  readonly onReminderFired: (
    listener: ReminderFiredListener,
  ) => () => void;
  readonly createReminder: (
    input: CreateReminderInput,
  ) => Promise<Reminder>;
  readonly updateReminder: (
    id: string,
    input: UpdateReminderInput,
  ) => Promise<Reminder>;
  readonly deleteReminder: (id: string) => Promise<boolean>;
  readonly getReminder: (id: string) => Promise<Reminder | null>;
  readonly listReminders: () => Promise<readonly Reminder[]>;
  readonly markReminderCompleted: (id: string) => Promise<Reminder>;
}

export interface PomodoroBridge {
  readonly startPomodoro: (durationMinutes: number) => Promise<void>;
  readonly notifyCustomPomodoroPanelClosed: () => void;
  readonly onCustomPomodoroDurationRequested: (
    listener: PomodoroCustomDurationRequestListener,
  ) => () => void;
  readonly getPomodoroState: () => PomodoroState | null;
  readonly onPomodoroStateChanged: (
    listener: PomodoroStateListener,
  ) => () => void;
  readonly onPomodoroCompleted: (
    listener: PomodoroCompletionListener,
  ) => () => void;
}

export interface CompanionBridge
  extends CompanionWindowBridge,
    CompanionSettingsBridge,
    ReminderBridge,
    PomodoroBridge {
  readonly platform: string;
  readonly onUserNamePanelRequested: (
    listener: UserNamePanelRequestListener,
  ) => () => void;
  readonly onStickyMessagePanelRequested: (
    listener: StickyMessagePanelRequestListener,
  ) => () => void;
  readonly onDailyPlannerPanelRequested: (
    listener: DailyPlannerPanelRequestListener,
  ) => () => void;
  readonly askAI: (request: AIConversationRequest) => Promise<AIAskResult>;
  readonly getDailyPlanner: () => Promise<DailyPlannerBriefing>;
}

export interface PreferencesBridge extends PreferencesSettingsBridge {
  readonly updateAiConfiguration: (
    configuration: AiConfigurationUpdate,
  ) => Promise<PreferencesSettings>;
  readonly listAIModels: () => Promise<AIModelListResult>;
  readonly testAIConnection: () => Promise<AIConnectionTestResult>;
  readonly getUpdateStatus: () => Promise<UpdateStatus>;
  readonly checkForUpdates: () => Promise<UpdateStatus>;
  readonly onUpdateStatusChanged: (
    listener: UpdateStatusListener,
  ) => () => void;
}

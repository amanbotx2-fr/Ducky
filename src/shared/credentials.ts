export const CREDENTIAL_IDS = ['aiApiKey'] as const;

export type CredentialId = (typeof CREDENTIAL_IDS)[number];

export type CredentialState =
  | 'configured'
  | 'missing'
  | 'requiresReentry';

/**
 * Secret-free credential metadata safe to return to a renderer.
 *
 * Plaintext retrieval is deliberately absent. Future native consumers obtain
 * secrets inside their owning runtime domain instead of crossing this bridge.
 */
export interface CredentialStatus {
  readonly id: CredentialId;
  readonly state: CredentialState;
}

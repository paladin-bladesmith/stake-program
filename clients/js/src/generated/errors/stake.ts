/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

/** AmountGreaterThanZero: Amount cannot be greater than zero */
export const STAKE_ERROR__AMOUNT_GREATER_THAN_ZERO = 0x0; // 0
/** InvalidTokenOwner: Invalid token owner */
export const STAKE_ERROR__INVALID_TOKEN_OWNER = 0x1; // 1
/** InvalidTransferHookProgramId: Invalid transfer hook program id */
export const STAKE_ERROR__INVALID_TRANSFER_HOOK_PROGRAM_ID = 0x2; // 2
/** InvalidAccountDataLength: Invalid account data length */
export const STAKE_ERROR__INVALID_ACCOUNT_DATA_LENGTH = 0x3; // 3
/** InvalidMint: Invalid mint */
export const STAKE_ERROR__INVALID_MINT = 0x4; // 4
/** MissingTransferHook: Missing transfer hook */
export const STAKE_ERROR__MISSING_TRANSFER_HOOK = 0x5; // 5
/** InvalidAuthority: Invalid authority */
export const STAKE_ERROR__INVALID_AUTHORITY = 0x6; // 6
/** AuthorityNotSet: Authority is not set */
export const STAKE_ERROR__AUTHORITY_NOT_SET = 0x7; // 7
/** CloseAuthorityNotNone: Close authority must be none */
export const STAKE_ERROR__CLOSE_AUTHORITY_NOT_NONE = 0x6; // 6
/** DelegateNotNone: Delegate must be none */
export const STAKE_ERROR__DELEGATE_NOT_NONE = 0x7; // 7
/** InvalidTokenAccountExtension: Invalid token account extension */
export const STAKE_ERROR__INVALID_TOKEN_ACCOUNT_EXTENSION = 0x8; // 8
/** MissingTokenAccountExtensions: Missing token account extensions */
export const STAKE_ERROR__MISSING_TOKEN_ACCOUNT_EXTENSIONS = 0x9; // 9

export type StakeError =
  | typeof STAKE_ERROR__AMOUNT_GREATER_THAN_ZERO
  | typeof STAKE_ERROR__AUTHORITY_NOT_SET
  | typeof STAKE_ERROR__CLOSE_AUTHORITY_NOT_NONE
  | typeof STAKE_ERROR__DELEGATE_NOT_NONE
  | typeof STAKE_ERROR__INVALID_ACCOUNT_DATA_LENGTH
  | typeof STAKE_ERROR__INVALID_AUTHORITY
  | typeof STAKE_ERROR__INVALID_MINT
  | typeof STAKE_ERROR__INVALID_TOKEN_ACCOUNT_EXTENSION
  | typeof STAKE_ERROR__INVALID_TOKEN_OWNER
  | typeof STAKE_ERROR__INVALID_TRANSFER_HOOK_PROGRAM_ID
  | typeof STAKE_ERROR__MISSING_TOKEN_ACCOUNT_EXTENSIONS
  | typeof STAKE_ERROR__MISSING_TRANSFER_HOOK;

let stakeErrorMessages: Record<StakeError, string> | undefined;
if (__DEV__) {
  stakeErrorMessages = {
    [STAKE_ERROR__AMOUNT_GREATER_THAN_ZERO]: `Amount cannot be greater than zero`,
    [STAKE_ERROR__AUTHORITY_NOT_SET]: `Authority is not set`,
    [STAKE_ERROR__CLOSE_AUTHORITY_NOT_NONE]: `Close authority must be none`,
    [STAKE_ERROR__DELEGATE_NOT_NONE]: `Delegate must be none`,
    [STAKE_ERROR__INVALID_ACCOUNT_DATA_LENGTH]: `Invalid account data length`,
    [STAKE_ERROR__INVALID_AUTHORITY]: `Invalid authority`,
    [STAKE_ERROR__INVALID_MINT]: `Invalid mint`,
    [STAKE_ERROR__INVALID_TOKEN_ACCOUNT_EXTENSION]: `Invalid token account extension`,
    [STAKE_ERROR__INVALID_TOKEN_OWNER]: `Invalid token owner`,
    [STAKE_ERROR__INVALID_TRANSFER_HOOK_PROGRAM_ID]: `Invalid transfer hook program id`,
    [STAKE_ERROR__MISSING_TOKEN_ACCOUNT_EXTENSIONS]: `Missing token account extensions`,
    [STAKE_ERROR__MISSING_TRANSFER_HOOK]: `Missing transfer hook`,
  };
}

export function getStakeErrorMessage(code: StakeError): string {
  if (__DEV__) {
    return (stakeErrorMessages as Record<StakeError, string>)[code];
  }

  return 'Error message not available in production bundles. Compile with `__DEV__` set to true to see more information.';
}

/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
export type ExecuteSaleParams = {
  escrowPaymentBump: number;
  freeTradeStateBump: number;
  sellerTradeStateBump: number;
  programAsSignerBump: number;
};

/**
 * @category userTypes
 * @category generated
 */
export const executeSaleParamsBeet = new beet.BeetArgsStruct<ExecuteSaleParams>(
  [
    ['escrowPaymentBump', beet.u8],
    ['freeTradeStateBump', beet.u8],
    ['sellerTradeStateBump', beet.u8],
    ['programAsSignerBump', beet.u8],
  ],
  'ExecuteSaleParams',
);

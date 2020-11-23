// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
import protos from './protobuf/protobuf.js';
import { signXOPayload } from './crypto.js';
import { MESSAGE_NAME, MESSAGE_VERSION, MESSAGE_PREFIX } from './addressing.js';
import { calculateNamespaceRegistryAddress, computeContractAddress, computeContractRegistryAddress, } from './addressing.js';
const crypto = require('crypto');
const { Transaction, TransactionHeader, Batch, BatchHeader, BatchList } = require('sawtooth-sdk/protobuf');
// The Sawtooth Sabre transaction family name (sabre)
const SABRE_FAMILY_NAME = 'sabre';
// The Sawtooth Sabre transaction family version (0.5)
const SABRE_FAMILY_VERSION = '0.5';
export function createTransaction(payloadBytes, inputs, outputs, user) {
    const excuteTransactionAction = protos.ExecuteContractAction.create({
        name: 'sawtooth_message',
        version: MESSAGE_VERSION,
        inputs,
        outputs,
        payload: payloadBytes,
    });
    const sabrePayload = protos.SabrePayload.encode({
        action: protos.SabrePayload.Action.EXECUTE_CONTRACT,
        executeContract: excuteTransactionAction,
    }).finish();
    const transactionHeaderBytes = TransactionHeader.encode({
        familyName: MESSAGE_NAME,
        familyVersion: MESSAGE_VERSION,
        inputs: prepare_inputs(inputs),
        outputs,
        signerPublicKey: user.publicKey,
        batcherPublicKey: user.publicKey,
        dependencies: [],
        payloadSha512: crypto.createHash('sha512').update(sabrePayload).digest('hex'),
    }).finish();
    const signature = signXOPayload(transactionHeaderBytes, user.privateKey);
    return Transaction.create({
        header: transactionHeaderBytes,
        headerSignature: signature,
        payload: sabrePayload,
    });
}
export function createBatch(transactions, user) {
    const transactionIds = transactions.map((txn) => txn.headerSignature);
    const batchHeaderBytes = BatchHeader.encode({
        signerPublicKey: user.publicKey,
        transactionIds,
    }).finish();
    const signature = signXOPayload(batchHeaderBytes, user.privateKey);
    const batch = Batch.create({
        header: batchHeaderBytes,
        headerSignature: signature,
        transactions,
    });
    const batchListBytes = BatchList.encode({
        batches: [batch],
    }).finish();
    return batchListBytes;
}
function prepare_inputs(contractAddresses) {
    const returnAddresses = [
        computeContractRegistryAddress(MESSAGE_NAME),
        computeContractAddress(MESSAGE_NAME, MESSAGE_VERSION),
        calculateNamespaceRegistryAddress(MESSAGE_PREFIX),
    ];
    return returnAddresses.concat(contractAddresses);
}
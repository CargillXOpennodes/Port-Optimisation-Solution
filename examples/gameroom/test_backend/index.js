const sjcl = require("sjcl");
const fs = require('fs');
const crypto = require('crypto');
import { calculateGameAddress } from "./utils/addressing.js";
import { createTransaction, createBatch } from "./utils/transactions.js";
const MESSAGE_PREFIX = "b01374";
import { signPayload } from "./utils/crypto.js";
var XMLHttpRequest = require("xmlhttprequest").XMLHttpRequest;

function hashSHA256(salt, data) {
    const out = sjcl.hash.sha256.hash(salt + data);
    return sjcl.codec.hex.fromBits(out);
}

function encrypt(password, privateKey) {
    return JSON.stringify(sjcl.encrypt(password, privateKey));
}

// console.log(hashSHA256("alice@cargill.com", "password"))
// console.log(toHex(encrypt("password", "9966b755baccc25e9d8bd9e8cd8a19fcf67953b2636d101e52f4a40473bb1ea7")))
// console.log(toHex("{\"iv\":\"y2Y+9CajZsnl0wyVT03OIg==\",\"v\":1,\"iter\":10000,\"ks\":128,\"ts\":64,\"mode\":\"ccm\",\"adata\":\"\",\"cipher\":\"aes\",\"salt\":\"o+C1NA76cIA=\",\"ct\":\"F51lsPG01gP3Hojo4UWHX8qZd/6t1uJer3FIhrOCMrqoQFsMT2G7l4sHi/vm6BPzqQgo/xx99xXDAL5oXs/j8C1ZEtePkltU\"}"))

// console.log("bob")
// console.log(hashSHA256("bob@cargill.com", "password"))
// console.log(toHex(encrypt("password", "c1e325f8508ee82f6d8c15649a8335549057523575b8cc603bd3f471645c2fad")))
// console.log(toHex("{\"iv\":\"y2Y+9CajZsnl0wyVT03OIg==\",\"v\":1,\"iter\":10000,\"ks\":128,\"ts\":64,\"mode\":\"ccm\",\"adata\":\"\",\"cipher\":\"aes\",\"salt\":\"o+C1NA76cIA=\",\"ct\":\"F51lsPG01gP3Hojo4UWHX8qZd/6t1uJer3FIhrOCMrqoQFsMT2G7l4sHi/vm6BPzqQgo/xx99xXDAL5oXs/j8C1ZEtePkltU\"}"))

const alice = {
  "email": "alice@cargill.com",
  "hashedPassword": "a12f5170c30d6d9504e6d1fc64f33b472cb8c69e904b66cb421889a8ff263ade",
  "publicKey": "037f92b94df1ff703031d100ea76777b6585b74182765627113020744e4bf3c895",
  "privateKey": "29ab144f5471f766d1de9c37bc6b7a35d638c6eaa41e71f219ea3f35eaa11170"
};
const bob = {
  "email": "bob@cargill.com",
  "hashedPassword": "4b6c8f7a8de9776aeb93e0bf4abf81666864b5b06cdb503b35383b0d02f30af0",
  "publicKey": "02d151f4389ba397cb87e0ca4bdba8012eeb0d457c3b6935830ce32e976a62d277",
  "privateKey": "dd8f92992971d26761b3bb8615c145c17050d4ed287e2b740d58388244c1d839"
};

const dan = {
  "privateKey": "b7b87a06cce430ba412368f646e934e7992733ba7aeda0ab6e418d524203ad4b"
}

async function http(
  method,
  url,
  data,
  headerFn,
  port
) {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open(method, `http://localhost:8002${url}`);
    if (headerFn) {
      headerFn(request);
    }
    request.onload = () => {
      if (request.status >= 200 && request.status < 300) {
        resolve(request.response);
      } else {
        const responseBody = JSON.parse(request.responseText);
        console.error(responseBody.message);
        if (request.status >= 400 && request.status < 500) {
          reject('Failed to send request. Contact the administrator for help.');
        } else {
          reject('The Gameroom server has encountered an error. Please contact the administrator.');
        }
      }
    };
    request.onerror = () => {
      console.error(request);
      reject('The Gameroom server has encountered an error. Please contact the administrator.');
    };
    request.send(data);
  });
}

async function submitBatch(payload, circuitID, port) {
  return await http(
    'POST', `/gamerooms/${circuitID}/batches`, payload, (request) => {
      request.setRequestHeader('Content-Type', 'application/octet-stream');
  }).catch((err) => {
    console.log("gi")
    console.log(err);
  }).then((rawBody) => {
    console.log("done");
    console.log(rawBody);
    // const jsonBody = JSON.parse(rawBody);
    // console.log(jsonBody.data);
    // return jsonBody.data;
  }, port);
}

function toHex(str) {
    var result = '';
    for (var i=0; i<str.length; i++) {
      result += str.charCodeAt(i).toString(16);
    }
    return result;
}

function createGame(user, gameName) {
    const payload = new Buffer(`${gameName},create,hello friend`, 'utf-8');
    const gameAdress = calculateGameAddress(gameName).slice(0, 6);
    const transaction = createTransaction(payload, [gameAdress], [gameAdress], user);
    return createBatch([transaction], user);
}

async function submitPayload(payload, port) {
  await http('POST', '/submit', payload, (request) => {
    request.setRequestHeader('Content-Type', 'application/octet-stream');
  }).catch((err) => {
    console.log(err)
  }, port)
}

const payload_bytes = [
  10,
  84,
  8,
  1,
  26,
  64,
  77,
  47,
  181,
  242,
  190,
  253,
  174,
  118,
  237,
  151,
  190,
  100,
  38,
  13,
  142,
  201,
  79,
  134,
  52,
  119,
  162,
  206,
  145,
  163,
  213,
  7,
  133,
  212,
  213,
  201,
  255,
  68,
  81,
  195,
  101,
  144,
  75,
  188,
  196,
  216,
  168,
  221,
  170,
  48,
  89,
  144,
  116,
  83,
  221,
  121,
  109,
  30,
  175,
  48,
  47,
  228,
  167,
  54,
  45,
  68,
  184,
  96,
  211,
  113,
  34,
  14,
  100,
  101,
  108,
  116,
  97,
  45,
  110,
  111,
  100,
  101,
  45,
  48,
  48,
  48,
  26,
  81,
  10,
  11,
  120,
  111,
  48,
  104,
  99,
  45,
  69,
  56,
  86,
  89,
  55,
  18,
  64,
  50,
  50,
  56,
  102,
  102,
  100,
  52,
  97,
  101,
  101,
  57,
  97,
  57,
  54,
  99,
  97,
  49,
  101,
  98,
  99,
  56,
  51,
  53,
  57,
  48,
  53,
  57,
  102,
  56,
  100,
  48,
  52,
  55,
  53,
  100,
  55,
  56,
  48,
  56,
  51,
  49,
  97,
  98,
  53,
  97,
  100,
  101,
  53,
  97,
  49,
  101,
  49,
  48,
  53,
  100,
  49,
  56,
  98,
  56,
  54,
  101,
  48,
  100,
  98,
  24,
  1
];

// submitPayload(signPayload(payload_bytes, alice.privateKey), 0).then((data) => console.log("Done"))
// submitPayload(signPayload(payload_bytes, bob.privateKey), 0).then((data) => console.log("Done"))
// submitPayload(signPayload(payload_bytes, dan.privateKey), 0).then((data) => console.log("Done"))

var payload1 = Buffer.from(signPayload(payload_bytes, bob.privateKey)).toString('base64')
console.log(payload1)

// fs.writeFile('../../../payload', createGame(alice, "name23"));//, "cGHwW-HydnS")

// console.log(Buffer.from(sdb).toString())
// console.log(createGame(alice, "first"))
// console.log(createGame(alice, "first").toString('binary'));

// submitBatch(createGame(alice, "first"), "vfp3v-R84nm").catch(err => console.log(err)).then(data => console.log(data));
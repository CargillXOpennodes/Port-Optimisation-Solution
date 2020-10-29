const sjcl = require("sjcl");
const crypto = require('crypto');
import { calculateGameAddress } from "./utils/addressing.js";
import { createTransaction, createBatch } from "./utils/transactions.js";
const MESSAGE_PREFIX = "f8daf5";
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
  "publicKey": "02685c1048fed717877ac4b9cf90f724c69a770d3c33a54e2e2483cb39beca8e2c",
  "privateKey": "c1e325f8508ee82f6d8c15649a8335549057523575b8cc603bd3f471645c2fad"
};
const bob = {
  "email": "bob@cargill.com",
  "hashedPassword": "4b6c8f7a8de9776aeb93e0bf4abf81666864b5b06cdb503b35383b0d02f30af0",
  "publicKey": "0317bd9b540436804fe8c2d0874188c708d9bc3909a03614e9b7b7a8c318de026e",
  "privateKey": "9966b755baccc25e9d8bd9e8cd8a19fcf67953b2636d101e52f4a40473bb1ea7"
};

async function http(
  method,
  url,
  data,
  headerFn,
  port
) {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open(method, `http://localhost:8001${url}`);
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
    const payload = new Buffer(`${gameName},create,`, 'utf-8');
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

const payload_bytes =   [
  10,
  84,
  8,
  1,
  26,
  64,
  106,
  40,
  81,
  47,
  46,
  62,
  98,
  132,
  181,
  73,
  5,
  247,
  187,
  36,
  203,
  241,
  236,
  145,
  210,
  123,
  221,
  170,
  166,
  185,
  160,
  143,
  233,
  101,
  157,
  90,
  21,
  19,
  77,
  169,
  246,
  145,
  3,
  147,
  148,
  26,
  205,
  23,
  161,
  133,
  76,
  158,
  96,
  46,
  6,
  49,
  142,
  186,
  107,
  24,
  174,
  217,
  127,
  52,
  239,
  112,
  230,
  105,
  0,
  23,
  34,
  14,
  98,
  117,
  98,
  98,
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
  97,
  106,
  84,
  65,
  102,
  45,
  85,
  108,
  54,
  100,
  72,
  18,
  64,
  99,
  48,
  49,
  55,
  98,
  54,
  56,
  97,
  50,
  99,
  98,
  99,
  51,
  98,
  54,
  56,
  98,
  57,
  102,
  52,
  100,
  100,
  50,
  57,
  57,
  97,
  56,
  100,
  99,
  100,
  55,
  52,
  51,
  52,
  54,
  102,
  53,
  54,
  99,
  54,
  48,
  56,
  100,
  98,
  56,
  99,
  98,
  97,
  52,
  57,
  55,
  51,
  98,
  55,
  48,
  54,
  50,
  50,
  54,
  55,
  49,
  53,
  49,
  101,
  24,
  1
];

// submitPayload(signPayload(payload_bytes, "9966b755baccc25e9d8bd9e8cd8a19fcf67953b2636d101e52f4a40473bb1ea7", "8001")).then((data) => console.log("fa"))
// submitPayload(signPayload(payload_bytes, alice.privateKey)).then((data) => console.log("fa"))

submitBatch(createGame(bob, "first"), "ajTAf-Ul6dH")
// console.log(createGame(alice, "first"))
// console.log(createGame(alice, "first").toString('binary'));

// submitBatch(createGame(alice, "first"), "vfp3v-R84nm").catch(err => console.log(err)).then(data => console.log(data));
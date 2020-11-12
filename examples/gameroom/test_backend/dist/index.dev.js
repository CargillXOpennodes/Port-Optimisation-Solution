"use strict";

var _addressing = require("./utils/addressing.js");

var _transactions = require("./utils/transactions.js");

var _crypto = require("./utils/crypto.js");

var sjcl = require("sjcl");

var fs = require('fs');

var crypto = require('crypto');

var MESSAGE_PREFIX = "b01374";

var XMLHttpRequest = require("xmlhttprequest").XMLHttpRequest;

function hashSHA256(salt, data) {
  var out = sjcl.hash.sha256.hash(salt + data);
  return sjcl.codec.hex.fromBits(out);
}

function encrypt(password, privateKey) {
  return JSON.stringify(sjcl.encrypt(password, privateKey));
} // console.log(hashSHA256("alice@cargill.com", "password"))
// console.log(toHex(encrypt("password", "9966b755baccc25e9d8bd9e8cd8a19fcf67953b2636d101e52f4a40473bb1ea7")))
// console.log(toHex("{\"iv\":\"y2Y+9CajZsnl0wyVT03OIg==\",\"v\":1,\"iter\":10000,\"ks\":128,\"ts\":64,\"mode\":\"ccm\",\"adata\":\"\",\"cipher\":\"aes\",\"salt\":\"o+C1NA76cIA=\",\"ct\":\"F51lsPG01gP3Hojo4UWHX8qZd/6t1uJer3FIhrOCMrqoQFsMT2G7l4sHi/vm6BPzqQgo/xx99xXDAL5oXs/j8C1ZEtePkltU\"}"))
// console.log("bob")
// console.log(hashSHA256("bob@cargill.com", "password"))
// console.log(toHex(encrypt("password", "c1e325f8508ee82f6d8c15649a8335549057523575b8cc603bd3f471645c2fad")))
// console.log(toHex("{\"iv\":\"y2Y+9CajZsnl0wyVT03OIg==\",\"v\":1,\"iter\":10000,\"ks\":128,\"ts\":64,\"mode\":\"ccm\",\"adata\":\"\",\"cipher\":\"aes\",\"salt\":\"o+C1NA76cIA=\",\"ct\":\"F51lsPG01gP3Hojo4UWHX8qZd/6t1uJer3FIhrOCMrqoQFsMT2G7l4sHi/vm6BPzqQgo/xx99xXDAL5oXs/j8C1ZEtePkltU\"}"))


var alice = {
  "email": "alice@cargill.com",
  "hashedPassword": "a12f5170c30d6d9504e6d1fc64f33b472cb8c69e904b66cb421889a8ff263ade",
  "publicKey": "037f92b94df1ff703031d100ea76777b6585b74182765627113020744e4bf3c895",
  "privateKey": "29ab144f5471f766d1de9c37bc6b7a35d638c6eaa41e71f219ea3f35eaa11170"
};
var bob = {
  "email": "bob@cargill.com",
  "hashedPassword": "4b6c8f7a8de9776aeb93e0bf4abf81666864b5b06cdb503b35383b0d02f30af0",
  "publicKey": "02d151f4389ba397cb87e0ca4bdba8012eeb0d457c3b6935830ce32e976a62d277",
  "privateKey": "dd8f92992971d26761b3bb8615c145c17050d4ed287e2b740d58388244c1d839"
};

function http(method, url, data, headerFn, port) {
  return regeneratorRuntime.async(function http$(_context) {
    while (1) {
      switch (_context.prev = _context.next) {
        case 0:
          return _context.abrupt("return", new Promise(function (resolve, reject) {
            var request = new XMLHttpRequest();
            request.open(method, "http://localhost:8001".concat(url));

            if (headerFn) {
              headerFn(request);
            }

            request.onload = function () {
              if (request.status >= 200 && request.status < 300) {
                resolve(request.response);
              } else {
                var responseBody = JSON.parse(request.responseText);
                console.error(responseBody.message);

                if (request.status >= 400 && request.status < 500) {
                  reject('Failed to send request. Contact the administrator for help.');
                } else {
                  reject('The Gameroom server has encountered an error. Please contact the administrator.');
                }
              }
            };

            request.onerror = function () {
              console.error(request);
              reject('The Gameroom server has encountered an error. Please contact the administrator.');
            };

            request.send(data);
          }));

        case 1:
        case "end":
          return _context.stop();
      }
    }
  });
}

function submitBatch(payload, circuitID, port) {
  return regeneratorRuntime.async(function submitBatch$(_context2) {
    while (1) {
      switch (_context2.prev = _context2.next) {
        case 0:
          _context2.next = 2;
          return regeneratorRuntime.awrap(http('POST', "/gamerooms/".concat(circuitID, "/batches"), payload, function (request) {
            request.setRequestHeader('Content-Type', 'application/octet-stream');
          })["catch"](function (err) {
            console.log("gi");
            console.log(err);
          }).then(function (rawBody) {
            console.log("done");
            console.log(rawBody); // const jsonBody = JSON.parse(rawBody);
            // console.log(jsonBody.data);
            // return jsonBody.data;
          }, port));

        case 2:
          return _context2.abrupt("return", _context2.sent);

        case 3:
        case "end":
          return _context2.stop();
      }
    }
  });
}

function toHex(str) {
  var result = '';

  for (var i = 0; i < str.length; i++) {
    result += str.charCodeAt(i).toString(16);
  }

  return result;
}

function createGame(user, gameName) {
  var payload = new Buffer("".concat(gameName, ",create,hello friend"), 'utf-8');
  var gameAdress = (0, _addressing.calculateGameAddress)(gameName).slice(0, 6);
  var transaction = (0, _transactions.createTransaction)(payload, [gameAdress], [gameAdress], user);
  return (0, _transactions.createBatch)([transaction], user);
}

function submitPayload(payload, port) {
  return regeneratorRuntime.async(function submitPayload$(_context3) {
    while (1) {
      switch (_context3.prev = _context3.next) {
        case 0:
          _context3.next = 2;
          return regeneratorRuntime.awrap(http('POST', '/submit', payload, function (request) {
            request.setRequestHeader('Content-Type', 'application/octet-stream');
          })["catch"](function (err) {
            console.log(err);
          }, port));

        case 2:
        case "end":
          return _context3.stop();
      }
    }
  });
}

var payload_bytes = [10, 84, 8, 1, 26, 64, 103, 246, 134, 39, 67, 20, 9, 145, 185, 226, 241, 69, 241, 30, 60, 144, 3, 83, 55, 170, 5, 95, 39, 34, 179, 39, 216, 174, 50, 217, 114, 139, 181, 58, 73, 245, 109, 68, 9, 219, 106, 39, 207, 174, 38, 240, 59, 211, 8, 238, 88, 187, 104, 95, 76, 74, 179, 149, 16, 69, 226, 0, 241, 54, 34, 14, 98, 117, 98, 98, 97, 45, 110, 111, 100, 101, 45, 48, 48, 48, 26, 81, 10, 11, 106, 72, 73, 108, 117, 45, 121, 69, 65, 76, 104, 18, 64, 51, 100, 54, 48, 54, 55, 54, 98, 102, 49, 48, 54, 55, 101, 98, 97, 55, 102, 48, 49, 101, 49, 50, 100, 48, 99, 57, 51, 55, 98, 53, 101, 50, 100, 55, 102, 48, 98, 55, 54, 49, 51, 50, 49, 51, 49, 49, 99, 56, 100, 55, 50, 57, 97, 56, 53, 98, 49, 100, 99, 97, 52, 53, 49, 24, 1];
submitPayload((0, _crypto.signPayload)(payload_bytes, bob.privateKey)).then(function (data) {
  return console.log("fa");
}); // fs.writeFile('../../../payload', createGame(alice, "name23"));//, "cGHwW-HydnS")
// console.log(Buffer.from(sdb).toString())
// console.log(createGame(alice, "first"))
// console.log(createGame(alice, "first").toString('binary'));
// submitBatch(createGame(alice, "first"), "vfp3v-R84nm").catch(err => console.log(err)).then(data => console.log(data));
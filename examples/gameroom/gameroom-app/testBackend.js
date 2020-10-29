const sjcl = require("sjcl");
const crypto = require('crypto');

function hashSHA256(salt, data) {
    const out = sjcl.hash.sha256.hash(salt + data);
    return sjcl.codec.hex.fromBits(out);
}

function encrypt(password, privateKey) {
    return JSON.stringify(sjcl.encrypt(password, privateKey));
}

console.log(hashSHA256("alice@cargill.com", "password"))
console.log(toHex(encrypt("password", "c1e325f8508ee82f6d8c15649a8335549057523575b8cc603bd3f471645c2fad")))
console.log(toHex("{\"iv\":\"y2Y+9CajZsnl0wyVT03OIg==\",\"v\":1,\"iter\":10000,\"ks\":128,\"ts\":64,\"mode\":\"ccm\",\"adata\":\"\",\"cipher\":\"aes\",\"salt\":\"o+C1NA76cIA=\",\"ct\":\"F51lsPG01gP3Hojo4UWHX8qZd/6t1uJer3FIhrOCMrqoQFsMT2G7l4sHi/vm6BPzqQgo/xx99xXDAL5oXs/j8C1ZEtePkltU\"}"))
function toHex(str) {
    var result = '';
    for (var i=0; i<str.length; i++) {
      result += str.charCodeAt(i).toString(16);
    }
    return result;
}

function calculateGameAddress(gameName) {
    const gameNameHash =  crypto.createHash('sha512').update(gameName).digest('hex');
    return `${MESSAGE_PREFIX}${gameNameHash.slice(0, 64)}`;
}

function createGame(gameName) {
    const user = {
        email: "alice@cargill.com",
        hashedPassword: hashSHA256("alice@cargill.com", "password")
    };
    const payload = new Buffer(`${gameName},create,`, 'utf-8');
    const gameAdress = calculateGameAddress(gameName);
    const transaction = createTransaction(payload, [gameAdress], [gameAdress], user);
    return createBatch([transaction], user);
}

console.log(createGame("first"))

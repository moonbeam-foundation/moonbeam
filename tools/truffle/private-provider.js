const ProviderEngine = require("web3-provider-engine");
const WalletSubprovider = require('web3-provider-engine/subproviders/wallet');
const RpcSubprovider = require('web3-provider-engine/subproviders/rpc');
const EthereumjsWallet = require('ethereumjs-wallet');



function ChainIdSubProvider(chainId) {
  this.chainId = chainId;
}

ChainIdSubProvider.prototype.setEngine = function(engine) {
  const self = this
  if (self.engine) return
  self.engine = engine
}
ChainIdSubProvider.prototype.handleRequest = function(payload, next, end) {
  if (payload.method == "eth_sendTransaction" && payload.params.length > 0 && typeof payload.params[0].chainId == "undefined") {
    payload.params[0].chainId = this.chainId;
  }
  next()
}


function NonceSubProvider() {
}

NonceSubProvider.prototype.setEngine = function(engine) {
  const self = this
  if (self.engine) return
  self.engine = engine
}
NonceSubProvider.prototype.handleRequest = function(payload, next, end) {
  if (payload.method == "eth_sendTransaction") {
    this.engine.sendAsync({
      jsonrpc:"2.0",
      id: Math.ceil(Math.random() * 4415011859092441),
      method: "eth_getTransactionCount",
      params: [payload.params[0].from, "latest"]
    }, (err, result) => {
      const nonce = typeof result.result == "string" ?
        result.result == "0x" ? 0 : parseInt(result.result.substring(2), 16) : 0;
      payload.params[0].nonce = nonce || 0;
      next();
    })
  } else {
    next()
  }
}

function PrivateKeyProvider(privateKey, providerUrl, chainId) {
  if (!privateKey) {
    throw new Error(`Private Key missing, non-empty string expected, got "${privateKey}"`);
  }

  if (!providerUrl) {
    throw new Error(`Provider URL missing, non-empty string expected, got "${providerUrl}"`);
  }

  this.wallet = EthereumjsWallet.default.fromPrivateKey(new Buffer(privateKey, "hex"));
  this.address = "0x" + this.wallet.getAddress().toString("hex");

  this.engine = new ProviderEngine({useSkipCache: false});
  
  this.engine.addProvider(new ChainIdSubProvider(chainId));
  this.engine.addProvider(new NonceSubProvider());
  this.engine.addProvider(new WalletSubprovider(this.wallet, {}));
  this.engine.addProvider(new RpcSubprovider({ rpcUrl: providerUrl }));

  this.engine.start();
  
}


PrivateKeyProvider.prototype.sendAsync = function(payload, callback) {
  return this.engine.sendAsync.apply(this.engine, arguments);
};

PrivateKeyProvider.prototype.send = function() {
  return this.engine.send.apply(this.engine, arguments);
};

module.exports = PrivateKeyProvider;
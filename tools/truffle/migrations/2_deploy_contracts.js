var MyToken = artifacts.require("MyToken");

module.exports = function (deployer) {
  // deployment steps
  deployer.deploy(MyToken, "8000000000000000000000000", { gas: 4294967295 });
};


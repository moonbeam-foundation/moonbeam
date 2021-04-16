export const contractSources: { [key: string]: string } = {
  // Solidity: contract test {function multiply(uint a) public pure returns(uint d) {return a * 7;}}
  TestContract: `
        pragma solidity >=0.8.0;
    
    contract TestContract {function multiply(uint a) public pure returns(uint d) {return a * 7;}}`,
  // simple incremental count contract to test contract with state changes
  TestContractIncr: `
      pragma solidity >=0.8.0;
      
      contract TestContractIncr {
          uint public count;
      
          constructor() public {
              count = 0;
          }
      
          function incr() public {
              count=count+1;
          }
      }`,
  // infinite loop call
  InfiniteContract: `
    pragma solidity >=0.8.0;
    
    contract InfiniteContract {
        function infinite() public pure returns(uint d) {while (true) {}}
    }`,
  // infinite loop call with variable alocation
  InfiniteContractVar: `
  pragma solidity >=0.8.0;
  
  contract InfiniteContractVar {
      uint public count;
  
      constructor() public {
          count = 0;
      }
  
      function infinite() public {
          while (true) {
              count=count+1;
          }
      }
  }`,
  // definite loop call with variable alocation
  FiniteLoopContract: `
    pragma solidity >=0.8.0;
    
    contract FiniteLoopContract {
        uint public count;
    
        constructor() public {
            count = 0;
        }
    
        function incr(uint n) public {
            uint i=0;
            while (i<n) {
                count=count+1;
                i+=1;
            }
        }
    }`,
};

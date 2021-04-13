export const contractSources: { [key: string]: string } = {
  // Solidity: contract test {function multiply(uint a) public pure returns(uint d) {return a * 7;}}
  TEST_CONTRACT: `
        pragma solidity >=0.8.0;
    
    contract TEST_CONTRACT {function multiply(uint a) public pure returns(uint d) {return a * 7;}}`,
  // simple incremental count contract to test contract with state changes
  TEST_CONTRACT_INCR: `
      pragma solidity >=0.8.0;
      
      contract TEST_CONTRACT_INCR {
          uint public count;
      
          constructor() public {
              count = 0;
          }
      
          function incr() public {
              count=count+1;
          }
      }`,
  // infinite loop call
  INFINITE_CONTRACT: `
    pragma solidity >=0.8.0;
    
    contract INFINITE_CONTRACT {
        function infinite() public pure returns(uint d) {while (true) {}}
    }`,
  // infinite loop call with variable alocation
  INFINITE_CONTRACT_VAR: `
  pragma solidity >=0.8.0;
  
  contract INFINITE_CONTRACT_VAR {
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
  FINITE_LOOP_CONTRACT: `
    pragma solidity >=0.8.0;
    
    contract FINITE_LOOP_CONTRACT {
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

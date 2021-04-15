export const contractSources: { [key: string]: string } = {
  // Solidity: contract test {function multiply(uint a) public pure returns(uint d) {return a * 7;}}
  Test_Contract: `
        pragma solidity >=0.8.0;
    
    contract Test_Contract {function multiply(uint a) public pure returns(uint d) {return a * 7;}}`,
  // simple incremental count contract to test contract with state changes
  Test_Contract_Incr: `
      pragma solidity >=0.8.0;
      
      contract Test_Contract_INCR {
          uint public count;
      
          constructor() public {
              count = 0;
          }
      
          function incr() public {
              count=count+1;
          }
      }`,
  // infinite loop call
  Infinite_Contract: `
    pragma solidity >=0.8.0;
    
    contract Infinite_Contract {
        function infinite() public pure returns(uint d) {while (true) {}}
    }`,
  // infinite loop call with variable alocation
  Infinite_Contract_Var: `
  pragma solidity >=0.8.0;
  
  contract Infinite_Contract_Var {
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
  Finite_Loop_Contract: `
    pragma solidity >=0.8.0;
    
    contract Finite_Loop_Contract {
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

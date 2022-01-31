// Script to make sure types are accurate

import _ from "underscore";
import { moonbeamDefinitions } from ".";
// import { moonbeamDefinitions_old } from "./index_old";

function logDif(obj1: any, obj2: any) {
  console.log("+different objects+");
  console.log(obj1);
  console.log(obj2);
}

// function to check types/json are the same
function deepEqual(obj1: any, obj2: any) {
  if (_.isArray(obj1)) {
    if (_.isArray(obj2)) {
      obj1.sort();
      obj2.sort();
      obj1.forEach((e, i) => {
        deepEqual(e, obj2[i]);
      });
    } else {
      logDif(obj1, obj2);
    }
  } else if (_.isObject(obj1)) {
    if (_.isObject(obj2)) {
      Object.keys(obj1).forEach((key) => {
        deepEqual(obj1[key], obj2[key]);
      });
    } else {
      logDif(obj1, obj2);
    }
  } else if (typeof obj1 === "string") {
    if (typeof obj2 === "string") {
      if (!(obj1 === obj2)) {
        logDif(obj1, obj2);
      }
    } else {
      logDif(obj1, obj2);
    }
  } else {
    console.log("unknown type", obj1);
  }
}

// Used to compare types with their older version to see missing fields
function compare(obj1: any, obj2: any) {
  let isSame = true;

  // check keys
  const keys1 = Object.keys(obj1);
  const keys2 = Object.keys(obj2);
  keys1.forEach((key) => {
    if (!keys2.includes(key)) {
      console.log(key + " is not included in obj2");
      isSame = false;
    } else {
      const sameValue = _.isMatch(obj2[key], obj1[key]);
      if (!sameValue) {
        console.log("different values for ", key);
        console.log(obj2[key]);
        console.log(obj1[key]);
        isSame = false;
      }
    }
  });
  keys2.forEach((key) => {
    if (!keys1.includes(key)) {
      console.log(key + " is not included in obj1");
      isSame = false;
    } else {
      const sameValue = _.isMatch(obj2[key], obj1[key]);
      if (!sameValue) {
        console.log("different values for ", key);
        console.log(obj2[key]);
        console.log(obj1[key]);
        isSame = false;
      }
    }
  });
  isSame = _.isMatch(obj1, obj2) && _.isMatch(obj2, obj1);
  console.log("isSame", isSame);
}
// compare(
//   moonbeamDefinitions_old.types ? moonbeamDefinitions_old.types[6].types : {},
//   moonbeamDefinitions_old.types ? moonbeamDefinitions_old.types[7].types : {}
// );
// compare(
//   moonbeamDefinitions.types ? moonbeamDefinitions.types[7].types : {},
//   moonbeamDefinitions_old.types ? moonbeamDefinitions_old.types[7].types : {}
// );

// Uncomment to compare different versions

// [0, 1, 2, 3, 4, 5, 6, 7].forEach((i) => {
//   deepEqual(
//     moonbeamDefinitions.types ? moonbeamDefinitions.types[i].types : {},
//     moonbeamDefinitions_old.types ? moonbeamDefinitions_old.types[i].types : {}
//   );
// });

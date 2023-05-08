#!/bin/bash

while IFS= read -r path
do
    echo "$path"
    # 
    matches=$(awk 'flag{
      if (buf ~ /}/ && buf !~ /reentrancy/) { 
        printf "(line %d) %s", line, buf; flag=0; buf=""        # single line declaration
      } else  {
          buf = buf $0 ORS;
          if (flag && /}/ && buf !~ /reentrancy/) {             # multiline declaration
            printf "(line %d) %s", line, buf; flag=0; buf="" 
          }
      }
    }
    /pallet-(ethereum|evm) = /{buf = $0 ORS; flag=1;line=NR}' "${path}") 

    if [[ -n "$matches" ]]; then
      echo "Check failed. Please add 'forbid-evm-reentrancy' feature to 'pallet-evm' and 'pallet-ethereum'."
      echo "${matches}"
      exit 1
    fi
done < <(find . -name "Cargo.toml" -not -path "*/target/*" -not -path "*/build/*")

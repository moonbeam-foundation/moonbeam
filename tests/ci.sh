
          POLKADOT_COMMIT=$(egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | head -1 | sed 's/.*#//' |  cut -c1-8)
          POLKADOT_REPO=$(egrep -o 'https.*/polkadot' Cargo.lock | head -1)
          DOCKER_TAG="purestake/polkadot-para-tests:sha-$POLKADOT_COMMIT"
          TEST_HASH=$(md5sum tests/package-lock.json | cut -f1 -d' ')
          TESTS_DOCKER_TAG="purestake/polkadot-para-tests:sha-$POLKADOT_COMMIT-${TEST_HASH:0:8}"
              
          
          #### Compute CPUS & CONCURRENCY
          CPU_PER_TEST=30;
          CPUS=$(lscpu  | grep '^CPU(s):' | grep -o '[0-9]*')
          CONCURRENCY=$((CPUS/CPU_PER_TEST))
          echo "Preparing $CONCURRENCY groups (CPUs/test: $CPU_PER_TEST, Total: $CPUS)"

          ####  Preparing the repository
          cd tests
          
          #### Splitting test files in groups (for concurrent execution)
          # TODO: remove grep for directories
          mapfile -t SPLIT < <(find tests -name "test-*.ts" | grep 'tests/.*/.*' |\
            while read -r file; do \
              L[$((COUNT%CONCURRENCY))]+="$file "; \
              echo ${L[$((COUNT%CONCURRENCY))]}; \
              COUNT=$(($COUNT + 1)); \
            done | tail -$CONCURRENCY )
            
          cd ..

          #### Cleaning possible previous data
          rm -f fail.txt docker*.txt
          
          #### Downloading image prior to concurrent execution
          docker pull $TESTS_DOCKER_TAG

          #### Script function to launch concurrent tests
          launch_as() {
            local cmd_name=$1
            echo "Starting $cmd_name" > $cmd_name.txt
            shift
            (time $@ || echo $cmd_name >> fail.txt) 2>&1 >> $cmd_name.txt
            echo "Done $cmd_name"
          }

          echo "Running tests inside docker: $TESTS_DOCKER_TAG"
          for i in $(eval echo "{0..$((CONCURRENCY - 1))}"); do
            launch_as docker$i docker run \
              --cpus $CPU_PER_TEST \
              -e BINARY_PATH='/moonbeam/build/moonbeam' \
              -e BASE_PATH_PREFIX='/ramdisk' \
              -e DEBUG='test:*' \
              -v $(pwd):/moonbeam:z \
              -u $UID \
              -w /ramdisk/tests \
              $TESTS_DOCKER_TAG \
                node node_modules/.bin/mocha --parallel -j 4 --exit -r ts-node/register "${SPLIT[$i]}" &
          done

          #### Waiting for all the group to finish
          wait
          echo "Done executing tests"
          
          #### Displaying output of each group
          for i in $(eval echo "{0..$((CONCURRENCY - 1))}"); do \
            cat docker$i.txt
          done
          egrep "[0-9][0-9]* (passing|pending|failing)" docker*.txt

          #### Forcing error if file is present
          if [ -f fail.txt ]; then
            cat fail.txt
            exit 1
          fi
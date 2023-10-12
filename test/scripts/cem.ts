import ""
import { ApiPromise } from "@polkadot/api"

async function main(){

    const api = await ApiPromise.create()

    api.tx.sudo.call
}

main()
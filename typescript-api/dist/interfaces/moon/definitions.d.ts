declare const _default: {
    types: {};
    rpc: {
        isBlockFinalized: {
            description: string;
            params: {
                name: string;
                type: string;
            }[];
            type: string;
        };
        isTxFinalized: {
            description: string;
            params: {
                name: string;
                type: string;
            }[];
            type: string;
        };
        getLatestSyncedBlock: {
            description: string;
            params: never[];
            type: string;
        };
    };
};
export default _default;

// exports.handler = async function (payload) {
    async function getContractOutput() {

        const {
            Keypair,
            Contract,
            SorobanRpc,
            TransactionBuilder,
            Networks,
            BASE_FEE,
            xdr,
            StrKey
        } = require("@stellar/stellar-sdk");
        const axios = require('axios');
        
 
    const sourceSecret = 'SCPRTX6CYO4P224AISRDOO3TCR7I5X7PBEHD2PUZGVS355WIPV33HT6Z';
    const sorobanRpcUrl = "https://soroban-testnet.stellar.org:443";
    const contractAddress = 'CCBBUNYF265KODZCM2M4A5JP6XWMQIK3ZU75PUZ7TKJ5XH3YSTRADX64';

    const url = `https://services-dev.0xauth.co/wallet/wallet-sdk/get-user-wallets?evm_address=0x34Be555065c984e4fb75d37D0b623F3388c7772b`;

    // try {
        const sourceKeypair = Keypair.fromSecret(sourceSecret);
        const server = new SorobanRpc.Server(sorobanRpcUrl);
        const contract = new Contract(contractAddress);

        const sourceAccount = await server.getAccount(sourceKeypair.publicKey());
        const response = await axios.get(url);
        const data = response.data;
        const walletAddress = data.data.stellar_address;

        console.log('Stellar Address:', walletAddress);

        const walletPublicKey = StrKey.decodeEd25519PublicKey(walletAddress);
        const walletAddressParam = xdr.ScAddress.scAddressTypeAccount(new xdr.PublicKey.publicKeyTypeEd25519(walletPublicKey));
        const walletParam = xdr.ScVal.scvAddress(walletAddressParam);

        const walletPublicKeySourcekey = StrKey.decodeEd25519PublicKey(sourceAccount.accountId());
        const walletAddressParamSource = xdr.ScAddress.scAddressTypeAccount(new xdr.PublicKey.publicKeyTypeEd25519(walletPublicKeySourcekey));
        const walletParamSourceKey = xdr.ScVal.scvAddress(walletAddressParamSource);

        let getIdentity = new TransactionBuilder(sourceAccount, {
            fee: BASE_FEE,
            networkPassphrase: Networks.TESTNET,
        })
            .addOperation(
                contract.call("get_identity", walletParam)
            )
            .setTimeout(30)
            .build();

            console.log(`get_identity builtTransactiongetIdentity=${getIdentity.toXDR()}`);

            console.log('Transaction builtgetIdentity:', getIdentity.toXDR());
            let prepareGetID = await server.prepareTransaction(getIdentity);

            prepareGetID.sign(sourceKeypair);
            let sendGetID = await server.sendTransaction(prepareGetID);
    
            let decodedAddress;
            if (sendGetID.status === "PENDING") {
                let getResponse;
                do {
                    console.log("Waiting for transaction confirmation...");
                    await new Promise((resolve) => setTimeout(resolve, 1000));
                    getResponse = await server.getTransaction(sendGetID.hash);
                } while (getResponse.status === "NOT_FOUND");
    
                if (getResponse.status === "SUCCESS") {
                    if (!getResponse.resultMetaXdr) {
                        throw new Error("Empty resultMetaXDR in getTransaction response");
                    }
                    let transactionMeta = getResponse.resultMetaXdr;
                    let returnValue = transactionMeta.v3().sorobanMeta().returnValue();
                    
                    const buffer = returnValue.value()._value;
                    
    
                    decodedAddress = StrKey.encodeContract(buffer); 
    
                    console.log('Returned address (decoded):', decodedAddress);
                } else {
                    throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
                }
            } else {
                throw new Error(sendGetID.errorResultXdr);
            }
            console.log('Sending AddKey transaction...');
            const identityAddress = decodedAddress;
            const identityAddressContract = new Contract(identityAddress);
            
            const purpose = xdr.ScVal.scvU32(3);
            const keyType = xdr.ScVal.scvU32(1);
            
            
            let addKey = new TransactionBuilder(sourceAccount, {
                fee: BASE_FEE,
                networkPassphrase: Networks.TESTNET,
            })
                .addOperation(
                    identityAddressContract.call("add_key", walletParamSourceKey, walletParamSourceKey, purpose, keyType)
    
                )
                .setTimeout(30)
                .build();
    
                console.log(`builtTransaction addKey=${addKey.toXDR()}`);

                console.log('Transaction addKey:', addKey.toXDR());
                let prepareAddKey = await server.prepareTransaction(addKey);
                console.log('Transaction prepared:', prepareAddKey);
    
                prepareAddKey.sign(sourceKeypair);

                let signedXDR = prepareAddKey.toEnvelope().toXDR("base64");
                console.log('Signed prepared transaction XDR:', signedXDR);


                let sendAddKey = await server.sendTransaction(prepareAddKey);
                console.log('Transaction sent:', JSON.stringify(sendAddKey));
        
        
            if (sendAddKey.status === "PENDING") {
                let getResponse;
                do {
                    console.log("Waiting for transaction confirmation...");
                    await new Promise((resolve) => setTimeout(resolve, 1000));
                    getResponse = await server.getTransaction(sendAddKey.hash);
                } while (getResponse.status === "NOT_FOUND");

                console.log('Transaction response:', JSON.stringify(getResponse));
                console.log(getResponse)
                if (getResponse.status === "SUCCESS") {
                    if (!getResponse.resultMetaXdr) {
                        throw new Error("Empty resultMetaXDR in getTransaction response");
                    }
                    let transactionMeta = getResponse.resultMetaXdr;
                    let returnValue = transactionMeta.v3().sorobanMeta().returnValue();
                    console.log('Transaction result:', returnValue.value());
                } else {
                    throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
                }
            } else {
                throw new Error(sendAddKey.errorResultXdr);
            }

            
};
getContractOutput();
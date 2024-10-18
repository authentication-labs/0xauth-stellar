// exports.handler = async function (payload) {
    async function getContractOutput() {
    // const main = async function (event) {

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
    const crypto = require('crypto');
 
    function generateSalt() {
        return crypto.randomBytes(32).toString('hex');
    }
    // const conditionRequest = payload.request?.body;
    // let evmWalletAddress;
    // // Check if conditionRequest exists and is valid
    // if (conditionRequest) {
    //     for (const key in conditionRequest) {
    //         if (conditionRequest.hasOwnProperty(key)) {
    //             if (key === 'matchReasons' && Array.isArray(conditionRequest[key])) {
    //                 conditionRequest[key].forEach((reason) => {
    //                     if (reason.args) {
    //                         console.log(`Match Reason Args: ${JSON.stringify(reason.args, null, 2)}`);
    //                         evmWalletAddress = reason.args[0];  // Assuming args[0] is the Ethereum address
    //                     }
    //                 });
    //             }
    //         }
    //     }
    // } else {
    //     console.warn('No condition request found in the payload.');
    //     return { status: 'error', message: 'No condition request found.' };
    // }

    // if (!evmWalletAddress) {
    //     console.error('No wallet address found in matchReasons.');
    //     return { status: 'error', message: 'No wallet address found.' };
    // }

    const sourceSecret = 'SCPRTX6CYO4P224AISRDOO3TCR7I5X7PBEHD2PUZGVS355WIPV33HT6Z';
    const sorobanRpcUrl = "https://soroban-testnet.stellar.org:443";
    const contractAddress = 'CCBBUNYF265KODZCM2M4A5JP6XWMQIK3ZU75PUZ7TKJ5XH3YSTRADX64';
    // URL to fetch data from
    const url = `https://services-dev.0xauth.co/wallet/wallet-sdk/get-user-wallets?evm_address=0x34Be555065c984e4fb75d37D0b623F3388c7772b`;

    try {
        const sourceKeypair = Keypair.fromSecret(sourceSecret);
        const server = new SorobanRpc.Server(sorobanRpcUrl);
        const contract = new Contract(contractAddress);

        const sourceAccount = await server.getAccount(sourceKeypair.publicKey());
        const response = await axios.get(url);
        const data = response.data;
        const walletAddress = data.data.stellar_address;
        console.log('Stellar Address:', walletAddress);
        // Define params for the create_identity function
        // Parameters
        const wasmHashHex = "a4e521074eb42b5309cf174f379154a7eff5b6bc0874baf3334513c7cdd33d9a";
        // const walletAddress = "GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB";
        const saltHex = generateSalt();
        console.log('Generated Salt:', saltHex);
        const initFunction = "initialize";
        const initArgs = [
            { type: "address", value: sourceAccount.accountId() }
            // Replace this with the correct xdr format for your arguments
        ];


        // Prepare parameters using ScVal and ScAddress
        const wasmHashParam = xdr.ScVal.scvBytes(Buffer.from(wasmHashHex, 'hex'));

        const walletPublicKey = StrKey.decodeEd25519PublicKey(walletAddress);
        const walletAddressParam = xdr.ScAddress.scAddressTypeAccount(new xdr.PublicKey.publicKeyTypeEd25519(walletPublicKey));
        const walletParam = xdr.ScVal.scvAddress(walletAddressParam);

        const saltParam = xdr.ScVal.scvBytes(Buffer.from(saltHex, 'hex'));
        const initFnParam = xdr.ScVal.scvSymbol(initFunction);

        // Prepare initArgs (conversion required to xdr format)
        const initArgsParams = xdr.ScVal.scvVec(
            initArgs.map(arg => {
                if (arg.type === "address") {
                    const publicKey = StrKey.decodeEd25519PublicKey(arg.value);
                    const scAddress = xdr.ScAddress.scAddressTypeAccount(new xdr.PublicKey.publicKeyTypeEd25519(publicKey));
                    return xdr.ScVal.scvAddress(scAddress);
                }
                throw new Error(`Unsupported argument type ${arg.type}`);
            })
        );

        let builtTransaction = new TransactionBuilder(sourceAccount, {
            fee: BASE_FEE,
            networkPassphrase: Networks.TESTNET,
        })
            .addOperation(
                contract.call("create_identity", wasmHashParam, walletParam, saltParam, initFnParam, initArgsParams)

            )
            .setTimeout(30)
            .build();
        console.log(`builtTransaction=${builtTransaction.toXDR()}`);

        console.log('Transaction built:', builtTransaction.toXDR());

        let preparedTransaction = await server.prepareTransaction(builtTransaction);
        console.log('Transaction prepared:', preparedTransaction);

        preparedTransaction.sign(sourceKeypair);

        let signedXDR = preparedTransaction.toEnvelope().toXDR("base64");
        console.log('Signed prepared transaction XDR:', signedXDR);

        let sendResponse = await server.sendTransaction(preparedTransaction);
        console.log('Transaction sent:', JSON.stringify(sendResponse));

        if (sendResponse.status === "PENDING") {
            let getResponse;
            do {
                console.log("Waiting for transaction confirmation...");
                await new Promise((resolve) => setTimeout(resolve, 1000));
                getResponse = await server.getTransaction(sendResponse.hash);
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
                console.log(returnValue.value().toString());
            } else {
                throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
            }
        } else {
            throw new Error(sendResponse.errorResultXdr);
        }
    } catch (err) {
        console.log(err)
        console.log("Sending transaction failed", JSON.stringify(err, null, 2));
    }
};

// main();
getContractOutput();
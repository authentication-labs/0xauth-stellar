const {
    Keypair,
    Contract,
    SorobanRpc,
    TransactionBuilder,
    Networks,
    BASE_FEE,
    xdr,
    StrKey,
    Address
} = require("@stellar/stellar-sdk");
const axios = require('axios');
async function getContractOutput() {
  
    const sourceSecret = 'SCPRTX6CYO4P224AISRDOO3TCR7I5X7PBEHD2PUZGVS355WIPV33HT6Z';
    const sorobanRpcUrl = "https://soroban-testnet.stellar.org:443";
    const contractAddress = 'CCBBUNYF265KODZCM2M4A5JP6XWMQIK3ZU75PUZ7TKJ5XH3YSTRADX64';

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

            let prepareGetID = await server.prepareTransaction(getIdentity);

            prepareGetID.sign(sourceKeypair);
            let sendGetID = await server.sendTransaction(prepareGetID);
            
            let buffer
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
                    
                    buffer = returnValue.value()._value; 
                    
                    decodedAddress = StrKey.encodeContract(buffer); 
 
                    console.log('Returned address (decoded):', decodedAddress);
                } else {
                    throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
                }
            } else {
                throw new Error(sendGetID.errorResultXdr);
            }
 
            const identityAddress = decodedAddress;
            const identityAddressContract = new Contract(identityAddress);
                
            const identityAddressBuffer = Buffer.from(identityAddressContract.address().toBuffer());
            const identityAddressParam = xdr.ScVal.scvAddress(
                xdr.ScAddress.scAddressTypeContract(identityAddressBuffer)
            ); 
 
            const topicBigInt = BigInt(42);
            const schemeBigInt = BigInt(1);
        
            console.log(`Converted topicBigInt: ${topicBigInt}, type: ${typeof topicBigInt}`);
            console.log(`Converted schemeBigInt: ${schemeBigInt}, type: ${typeof schemeBigInt}`);
        
            // Define constants for bit shifts
            const BIT_SHIFT_64 = 64n;
        
            // Extract 64-bit parts for topic
            const topicLoLo = topicBigInt & ((1n << BIT_SHIFT_64) - 1n);
            const topicLoHi = (topicBigInt >> BIT_SHIFT_64) & ((1n << BIT_SHIFT_64) - 1n);
            const topicHiLo = (topicBigInt >> (2n * BIT_SHIFT_64)) & ((1n << BIT_SHIFT_64) - 1n);
            const topicHiHi = (topicBigInt >> (3n * BIT_SHIFT_64)) & ((1n << BIT_SHIFT_64) - 1n);
        
            // Extract 64-bit parts for scheme
            const schemeLoLo = schemeBigInt & ((1n << BIT_SHIFT_64) - 1n);
            const schemeLoHi = (schemeBigInt >> BIT_SHIFT_64) & ((1n << BIT_SHIFT_64) - 1n);
            const schemeHiLo = (schemeBigInt >> (2n * BIT_SHIFT_64)) & ((1n << BIT_SHIFT_64) - 1n);
            const schemeHiHi = (schemeBigInt >> (3n * BIT_SHIFT_64)) & ((1n << BIT_SHIFT_64) - 1n);
        
            console.log(`topic parts: ${topicHiHi}, ${topicHiLo}, ${topicLoHi}, ${topicLoLo}`);
            console.log(`scheme parts: ${schemeHiHi}, ${schemeHiLo}, ${schemeLoHi}, ${schemeLoLo}`);
        
            // Create UInt256Parts instances using xdr utilities
            const topicParam = xdr.ScVal.scvU256(new xdr.UInt256Parts({
                hiHi: xdr.Uint64.fromString(topicHiHi.toString()),
                hiLo: xdr.Uint64.fromString(topicHiLo.toString()),
                loHi: xdr.Uint64.fromString(topicLoHi.toString()),
                loLo: xdr.Uint64.fromString(topicLoLo.toString())
            }));
        
            const schemeParam = xdr.ScVal.scvU256(new xdr.UInt256Parts({
                hiHi: xdr.Uint64.fromString(schemeHiHi.toString()),
                hiLo: xdr.Uint64.fromString(schemeHiLo.toString()),
                loHi: xdr.Uint64.fromString(schemeLoHi.toString()),
                loLo: xdr.Uint64.fromString(schemeLoLo.toString())
            }));
        
            console.log(`topicParam: ${topicParam}, schemeParam: ${schemeParam}`);
         
        const signatureParam = xdr.ScVal.scvBytes(Buffer.from('data_example', 'utf-8'));
        const dataParam = xdr.ScVal.scvBytes(Buffer.from('data_example', 'utf-8'));
        const uriParam = xdr.ScVal.scvBytes(Buffer.from('uri_example', 'utf-8'));
        console.log('Signature:', signatureParam);
    
        let customTransaction = new TransactionBuilder(sourceAccount, {
            fee: BASE_FEE,
            networkPassphrase: Networks.TESTNET,
        })
            .addOperation(
                identityAddressContract.call(
                    "add_claim",
                    walletParamSourceKey, // sender
                    topicParam,
                    schemeParam,
                    walletParam, // issuer
                    identityAddressParam, // issuer_wallet
                    signatureParam,
                    dataParam,
                    uriParam
                )
            )
            .setTimeout(30)
            .build();

        // Prepare, sign, and send the transaction
        let preparedTransaction = await server.prepareTransaction(customTransaction);
        console.log('Transaction prepared:', preparedTransaction);

        preparedTransaction.sign(sourceKeypair);
        let sendTransaction = await server.sendTransaction(preparedTransaction);
        console.log('Transaction sent:', JSON.stringify(sendTransaction));

        // Check for pending status and wait for confirmation
        if (sendTransaction.status === "PENDING") {
            let getResponse;
            do {
                console.log("Waiting for transaction confirmation...");
                await new Promise((resolve) => setTimeout(resolve, 1000));
                getResponse = await server.getTransaction(sendTransaction.hash);
            } while (getResponse.status === "NOT_FOUND");

            // Handle the confirmed transaction
            console.log('Transaction response:', JSON.stringify(getResponse));
            if (getResponse.status === "SUCCESS") {
                // Optional: Handle the return value if any
                if (getResponse.resultMetaXdr) {
                    let transactionMeta = getResponse.resultMetaXdr;
                    let returnValue = transactionMeta.v3().sorobanMeta().returnValue();
                    console.log('Transaction result:', returnValue.value());

                    // Decode returnValue if necessary
                    // Handle based on specific function return type
                } else {
                    throw new Error("Empty resultMetaXDR in getTransaction response");
                }
            } else {
                throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
            }
        } else {
            throw new Error(sendTransaction.errorResultXdr);
        }


    } catch (err) {
        console.error("An error occurred:", err);
        console.log("Sending transaction failed", JSON.stringify(err, null, 2));
    }




};
 

  getContractOutput();
 
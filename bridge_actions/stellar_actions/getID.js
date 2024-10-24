const StellarSdk = require('stellar-sdk');
const server = new StellarSdk.Server('https://horizon-testnet.stellar.org');

// Replace with the source secret key of the account invoking the contract
const aliceSecret = 'your_source_account_secret_key';
const aliceKeypair = StellarSdk.Keypair.fromSecret(aliceSecret);

// Replace with your contract ID and wallet address
const contractId = 'CCW5VNZWT3WUVPROFORFEOHVVC32RBNIJ6PEMJ7J7TCV4X5PQULAAAGZ';
const walletAddress = 'GBMZ7O5CB3Q2NQYVNEI3XZORKADJFMDFHUYYWFN72NADW3R5KCKP2JHJ';

// Address of the wallet for which identity is being queried
const wallet = StellarSdk.Address.fromString(walletAddress);

async function getContractOutput() {
    const account = await server.loadAccount(aliceKeypair.publicKey());

    // Build the transaction to invoke the contract
    const fee = await server.fetchBaseFee();

    const tx = new StellarSdk.TransactionBuilder(account, { fee, networkPassphrase: StellarSdk.Networks.TESTNET })
        .addOperation(StellarSdk.Operation.invokeHostFunction({
            hostFunction: StellarSdk.xdr.HostFunction.hostFunctionTypeInvokeContract(),
            // Assuming single argument 'wallet', packaged as expected by the contract
            args: [
                // Convert the wallet address to a Stellar address
                new StellarSdk.Address(wallet),
            ],
            // Set the contract ID
            contractId: new StellarSdk.ContractID(contractId),
        }))
        .setTimeout(30)
        .build();

    // Sign the transaction
    tx.sign(aliceKeypair);

    try {
        // Submit the transaction to the network
        const txResult = await server.submitTransaction(tx);
        console.log('Transaction Success:', txResult);
        
        // Extract contract output from the transaction result
        const identity = getContractResult(txResult);
        console.log('Contract returned identity:', identity);
        return identity;
    } catch (error) {
        console.error('Transaction Failed:', error.response.data);
        throw error;
    }
}

// Function to extract and parse contract result from transaction response
function getContractResult(txResult) {
    // Decode XDR to get the raw transaction result
    const buffer = Buffer.from(txResult.result_xdr, 'base64');
    const txResultXdr = StellarSdk.xdr.TransactionResult.fromXDR(buffer);
    
    // Locate the appropriate result related to the contract invocation
    for (const res of txResultXdr.result().results()) {
        if (res.tr().invokeHostFunctionResult() && res.tr().invokeHostFunctionResult().success()) {
            const contractResult = res.tr().invokeHostFunctionResult().success().results()[0];
            const returnedAddress = StellarSdk.Address.fromXDR(contractResult);
            return returnedAddress.toString();
        }
    }

    return null;
}

(async () => {
    const returnedIdentity = await getContractOutput();
    console.log('Stored Identity:', returnedIdentity);
})();
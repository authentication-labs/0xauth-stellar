exports.handler = async function (payload) {
    try {
        // Check if the payload is actually present
        if (!payload) {
            console.error('Payload is null or undefined');
            return { status: 'error', message: 'Payload is null or undefined' };
        }

        // Log the entire payload to see its structure
        console.log('Full Payload:', JSON.stringify(payload, null, 2));

        // Extract the condition request from the payload
        const conditionRequest = payload.request?.body;
        
        // Check if the condition request is actually present
        if (!conditionRequest) {
            console.error('No condition request found in payload.request.body');
            return { status: 'error', message: 'No condition request found in payload.request.body' };
        }

        // Log the condition request for debugging
        console.log('Condition Request:', JSON.stringify(conditionRequest, null, 2));

        // Inspect the structure inside the conditionRequest
        console.log('Inspecting Condition Request:', conditionRequest);
        for (const key in conditionRequest) {
            if (conditionRequest.hasOwnProperty(key)) {
                console.log(`${key}: ${JSON.stringify(conditionRequest[key], null, 2)}`);
            }
        }

        // Extract and log parts according to the Monitor Event Schema
        // Transaction details
        const transaction = conditionRequest.transaction;
        console.log('Transaction:', JSON.stringify(transaction, null, 2));

        // Block Hash
        const blockHash = conditionRequest.blockHash;
        console.log('Block Hash:', blockHash);

        // Match Reasons
        const matchReasons = conditionRequest.matchReasons;
        if (Array.isArray(matchReasons)) {
            console.log('Match Reasons:', JSON.stringify(matchReasons, null, 2));
            matchReasons.forEach((reason, index) => {
                console.log(`Match Reason #${index + 1}:`, JSON.stringify(reason, null, 2));
            });
        } else {
            console.log('No match reasons found or not an array.');
        }

        // Matched Addresses
        const matchedAddresses = conditionRequest.matchedAddresses;
        if (Array.isArray(matchedAddresses)) {
            console.log('Matched Addresses:', JSON.stringify(matchedAddresses, null, 2));
        } else {
            console.log('No matched addresses found or not an array.');
        }

        // Monitor details
        const monitor = conditionRequest.monitor;
        console.log('Monitor Details:', JSON.stringify(monitor, null, 2));

        // Value of the transaction
        const value = conditionRequest.value;
        console.log('Transaction Value:', value);

        // Metadata (if any)
        const metadata = conditionRequest.metadata;
        console.log('Metadata:', JSON.stringify(metadata, null, 2));

        // Return success after logging
        return { status: 'success', message: 'Monitor event logged and processed successfully.' };
    } catch (error) {
        console.error('Error processing the payload:', error);
        return { status: 'error', message: error.message };
    }
};





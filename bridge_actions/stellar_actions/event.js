exports.handler = async function (payload) {
    try {
        // Extract the condition request from the payload
        const conditionRequest = payload.request?.body;

        // Check if conditionRequest exists and is valid
        if (conditionRequest) {
            for (const key in conditionRequest) {
                if (conditionRequest.hasOwnProperty(key)) {
                    console.log("-----------------");
                    if (key === 'matchReasons' && Array.isArray(conditionRequest[key])) {
                        conditionRequest[key].forEach((reason, index) => {
                            console.log(`matchReasons[${index}]: ${JSON.stringify(reason, null, 2)}`);
                            if (reason.args) {
                                console.log(`  args: ${JSON.stringify(reason.args, null, 2)}`);
                            }
                            if (reason.params) {
                                console.log(`  params: ${JSON.stringify(reason.params, null, 2)}`);
                            }
                        });
                    }
                }
            }
        } else {
            console.warn('No condition request found in the payload.');
        }

        // Return a result
        return { status: 'success', message: 'Monitor event logged and processed successfully.' };
    } catch (error) {
        console.error('Error processing the payload:', error);
        return { status: 'error', message: error.message };
    }
};
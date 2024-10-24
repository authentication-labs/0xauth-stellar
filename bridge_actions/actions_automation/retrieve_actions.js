const { Defender } = require('@openzeppelin/defender-sdk');

// Replace with your actual API key and secret
const creds = { 
  apiKey: process.env.API_KEY, 
  apiSecret: process.env.API_SECRET 
};
const client = new Defender(creds);

async function listActions() {
  try {
    const response = await client.action.list();
    console.log('Full response:', JSON.stringify(response, null, 2)); // Log the full response

    // Assuming the actions are within a property called 'items' or similar
    const actions = response.items || []; // Adjust this line based on the actual structure

    if (Array.isArray(actions)) {
      actions.forEach(action => {
        console.log(`Action Name: ${action.name}`);
        
        // Check if the trigger is defined and log it as a JSON string
        if (action.trigger) {
          console.log('Trigger:', JSON.stringify(action.trigger, null, 2)); // Pretty print the trigger object
        } else {
          console.log('Trigger is undefined');
        }
      });
    } else {
      console.log('Actions is not an array. Check the structure of the response.');
    }
  } catch (error) {
    console.error('Error listing actions:', error);
  }
}

listActions();

const { Defender } = require('@openzeppelin/defender-sdk');

// Replace with your actual API key and secret
const creds = { 
  apiKey: process.env.API_KEY, 
  apiSecret: process.env.API_SECRET 
};
const client = new Defender(creds);

async function listMonitors() {
  try {
    const monitors = await client.monitor.list(); // Hypothetical method
    console.log('Monitors:', monitors);
  } catch (error) {
    console.error('Error listing monitors:', error);
  }
}

listMonitors();

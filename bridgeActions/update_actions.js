const fs = require('fs');
const path = require('path');
const archiver = require('archiver');
const { Defender } = require('@openzeppelin/defender-sdk');

const creds = { 
  apiKey: process.env.API_KEY, 
  apiSecret: process.env.API_SECRET 
};

const client = new Defender(creds);

const actions = [
  {
    name: 'addClaimStellar',
    autotaskId: '3d2811bd-33c5-4250-b6b3-ae06bdf8bd5c',
    actionId: '3d2811bd-33c5-4250-b6b3-ae06bdf8bd5c',
    bundledCodePath: path.resolve(__dirname, 'dist', 'addClaimStellar.bundle.js'),
    dependenciesVersion: 'v2024-08-21',
    monitorId: 'adcbba2b-a8ad-476e-b75e-845d754c1462', 
  },
  {
    name: 'removeClaimAction',
    autotaskId: '717b8a80-cb0c-47c1-bcb6-64f9aff5b6c7',
    actionId: '717b8a80-cb0c-47c1-bcb6-64f9aff5b6c7',
    bundledCodePath: path.resolve(__dirname, 'dist', 'removeClaimStellar.bundle.js'),
    dependenciesVersion: 'v2024-10-02',
    monitorId: '8c39964f-169f-4856-b7fd-7a9ec3a4e2c1', 
  },
  {
    name: 'create_identity&addKey action',
    autotaskId: '7a9f8f2d-784c-44ae-abd2-5b356718b355',
    actionId: '7a9f8f2d-784c-44ae-abd2-5b356718b355',
    bundledCodePath: path.resolve(__dirname, 'dist', 'createIdentityAddKey.bundle.js'),
    dependenciesVersion: 'v2024-08-05',
    monitorId: '418f043c-6061-4e68-9c18-91b30e3c5abe', 
  },
  {
    name: 'addKeyStellar',
    autotaskId: '8f1433e0-87e0-4d7c-8349-508f699ae834',
    actionId: '8f1433e0-87e0-4d7c-8349-508f699ae834',
    bundledCodePath: path.resolve(__dirname, 'dist', 'addKeyStellar.bundle.js'),
    dependenciesVersion: 'v2024-08-21',
    monitorId: '07ad5984-70f1-4bfc-bfe1-23b29064f50d', 
  },
  {
    name: 'removeKeyAction',
    autotaskId: 'e6420d46-8a93-41ca-8efd-d647f2ee4c1c',
    actionId: 'e6420d46-8a93-41ca-8efd-d647f2ee4c1c',
    bundledCodePath: path.resolve(__dirname, 'dist', 'removeKeyStellar.bundle.js'),
    dependenciesVersion: 'v2024-10-02',
    monitorId: '7b484657-1bd3-4b9e-90b3-da1fbfc26cdf', 
  },
];

async function zipAndEncode(filePath) {
    return new Promise((resolve, reject) => {
      const output = fs.createWriteStream(`${filePath}.zip`);
      const archive = archiver('zip', { zlib: { level: 9 } });
  
      output.on('close', () => {
        const zippedCode = fs.readFileSync(`${filePath}.zip`);
        resolve(zippedCode.toString('base64'));
      });
  
      archive.on('error', (err) => {
        reject(err);
      });
  
      archive.pipe(output);
      archive.file(filePath, { name: path.basename(filePath) });
      archive.finalize();
    });
  }
  
async function updateDefenderAction(action) {
  const encodedZippedCode = await zipAndEncode(action.bundledCodePath);

  const body = {
    autotaskId: action.autotaskId,
    actionId: action.actionId,
    name: action.name,
    encodedZippedCode,
    trigger: {
        type: 'sentinel', 
        cron: '*/99999999999999999 * * * *',
        frequencyMinutes: 0, 
      },
    paused: false,
    dependenciesVersion: action.dependenciesVersion,
  };

  try {
    const result = await client.action.update(body);
    console.log(`Action ${action.name} updated successfully`);
  } catch (error) {
    console.error(`Error updating action ${action.name}:`, error);
  }
}

async function updateAllActions() {
  for (const action of actions) {
    await updateDefenderAction(action);
  }
}

updateAllActions();
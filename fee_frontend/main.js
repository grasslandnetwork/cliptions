// Configuration - you'll need to get a project ID from WalletConnect Cloud
const PROJECT_ID = 'YOUR_PROJECT_ID_HERE'; // TODO: Replace with actual project ID

// Web3Modal configuration
const { Web3Modal } = window.Web3Modal;

const web3Modal = new Web3Modal({
    projectId: PROJECT_ID,
    standaloneChains: ['eip155:1', 'eip155:8453'] // Ethereum mainnet and Base
});

// State management
let provider = null;
let signer = null;
let userAddress = null;

// DOM elements
const connectButton = document.getElementById('connectWallet');
const twitterInput = document.getElementById('twitterHandle');
const statusDiv = document.getElementById('status');

// Event listeners
connectButton.addEventListener('click', handleConnectWallet);

function showStatus(message, type = 'info') {
    statusDiv.textContent = message;
    statusDiv.className = `status ${type}`;
    statusDiv.classList.remove('hidden');
}

function hideStatus() {
    statusDiv.classList.add('hidden');
}

async function handleConnectWallet() {
    try {
        // Validate Twitter handle
        const twitterHandle = twitterInput.value.trim();
        if (!twitterHandle) {
            showStatus('Please enter your Twitter username', 'error');
            return;
        }

        // Basic validation for Twitter handle
        if (!/^[a-zA-Z0-9_]{1,15}$/.test(twitterHandle)) {
            showStatus('Please enter a valid Twitter username (letters, numbers, and underscores only)', 'error');
            return;
        }

        connectButton.disabled = true;
        showStatus('Connecting to wallet...', 'info');

        // Connect wallet
        await connectWallet();
        
        if (!userAddress) {
            throw new Error('Failed to get wallet address');
        }

        showStatus('Wallet connected! Please sign the message to verify your identity...', 'info');

        // Create message to sign
        const message = `I am ${twitterHandle} on Twitter and I own this wallet address: ${userAddress}`;
        
        // Sign message
        showStatus('Please sign the message in your wallet...', 'info');
        const signature = await signMessage(message);

        // Send to backend
        showStatus('Verifying signature...', 'info');
        await submitToBackend(twitterHandle, userAddress, message, signature);

        showStatus('✅ Success! Your fee payment has been verified.', 'success');
        connectButton.textContent = 'Fee Paid Successfully';

    } catch (error) {
        console.error('Error:', error);
        showStatus(`Error: ${error.message}`, 'error');
        connectButton.disabled = false;
    }
}

async function connectWallet() {
    try {
        // Open wallet selection modal
        const { uri, approval } = await web3Modal.openModal();
        
        if (uri) {
            // Wait for wallet connection
            const session = await approval();
            
            if (session && session.namespaces && session.namespaces.eip155) {
                const accounts = session.namespaces.eip155.accounts;
                if (accounts && accounts.length > 0) {
                    // Extract address from the account string (format: "eip155:1:0x...")
                    userAddress = accounts[0].split(':')[2];
                    web3Modal.closeModal();
                    return;
                }
            }
        }
        
        throw new Error('Failed to connect wallet');
        
    } catch (error) {
        web3Modal.closeModal();
        throw error;
    }
}

async function signMessage(message) {
    try {
        // For this simplified version, we'll use a mock signature
        // In a real implementation, you'd use the wallet's signing capability
        // through the WalletConnect session
        
        // This is a placeholder - in practice you'd use the actual wallet connection
        // to sign the message. For now, we'll simulate it.
        const mockSignature = `0x${'a'.repeat(130)}`; // Mock signature
        
        // In a real implementation, this would be something like:
        // const signature = await signer.signMessage(message);
        
        return mockSignature;
        
    } catch (error) {
        throw new Error(`Failed to sign message: ${error.message}`);
    }
}

async function submitToBackend(twitterHandle, walletAddress, message, signature) {
    try {
        const response = await fetch('/verify-payment', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                twitter_handle: twitterHandle,
                wallet_address: walletAddress,
                message: message,
                signature: signature
            })
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
            throw new Error(errorData.error || `HTTP ${response.status}`);
        }

        const result = await response.json();
        return result;

    } catch (error) {
        throw new Error(`Failed to verify payment: ${error.message}`);
    }
}

// Initialize the app
document.addEventListener('DOMContentLoaded', function() {
    // Check if we have a valid project ID
    if (PROJECT_ID === 'YOUR_PROJECT_ID_HERE') {
        showStatus('⚠️ Demo mode: WalletConnect project ID not configured. Wallet connection will be simulated.', 'info');
    }
    
    hideStatus();
}); 
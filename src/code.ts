import { buildEnergyViz } from "./builder";

console.log('=== PLUGIN STARTING ===');
console.log('Plugin ID:', figma.pluginId);
console.log('Editor Type:', figma.editorType);

// __html__ will be replaced with HTML content during build
console.log('Calling figma.showUI...');
figma.showUI(__html__, { width: 600, height: 300 });
console.log('✓ UI shown successfully');

console.log('Setting up onmessage handler...');
figma.ui.onmessage = (msg) => {
    console.log('=== MESSAGE RECEIVED ===');
    console.log('Message type:', msg.type);
    console.log('Full message:', JSON.stringify(msg, null, 2));
    
    if (msg.type === 'analysis-ready') {
      console.log('Processing analysis-ready message...');
      console.log('Data received:', msg.data);
      console.log('Normalized array length:', msg.data && msg.data.normalized ? msg.data.normalized.length : 'undefined');
    
    try {
      buildEnergyViz(msg.data);
      figma.notify(`Received ${msg.data.normalized.length} packets`);
      console.log('✓ Visualization created successfully');
    } catch (error) {
      console.error('✗ Error creating visualization:', error);
      figma.notify('Error creating visualization');
    }
  } else {
    console.log('Unknown message type:', msg.type);
  }
};
console.log('✓ Message handler set up');

console.log('=== PLUGIN INITIALIZED ===');
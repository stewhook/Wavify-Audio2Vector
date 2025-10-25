export function buildEnergyViz(data: { normalized: number[]; minAvg: number; maxAvg: number }) {
    console.log('=== buildEnergyViz CALLED ===');
    console.log('Data:', data);
    console.log('Normalized values:', data.normalized);
    console.log('Min Avg:', data.minAvg);
    console.log('Max Avg:', data.maxAvg);
    
    try {
        console.log('Creating frame...');
        const frame = figma.createFrame();
        frame.name = "Audio Energy (Local)";
        frame.resize(600, 200);
        console.log('✓ Frame created:', frame.id);
        
        // Add some visualization based on the data
        if (data && data.normalized && data.normalized.length > 0) {
            console.log('Adding visualization elements...');
            const barWidth = 600 / data.normalized.length;
            
            for (let i = 0; i < data.normalized.length; i++) {
                const bar = figma.createRectangle();
                const height = data.normalized[i] * 180;
                bar.resize(barWidth - 2, height);
                bar.x = i * barWidth;
                bar.y = 200 - height;
                bar.fills = [{ type: 'SOLID', color: { r: 0.2, g: 0.6, b: 1 } }];
                frame.appendChild(bar);
            }
            console.log(`✓ Created ${data.normalized.length} bars`);
        }
        
        console.log('✓ Visualization complete');
    } catch (error) {
        console.error('✗ Error in buildEnergyViz:', error);
        throw error;
    }
}
  
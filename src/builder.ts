export function buildEnergyViz(data: { audioName: string, normalized: number[]; minAvg: number; maxAvg: number; resultPreference: string; }) {
    console.log('=== buildEnergyViz CALLED ===');
    console.log('Data:', data);
    console.log('Normalized values:', data.normalized);
    console.log('Min Avg:', data.minAvg);
    console.log('Max Avg:', data.maxAvg);
    
    try {
        console.log('Creating frame...');
        const frame = figma.createFrame();
        frame.fills = [];
        frame.name = data.audioName;
        frame.resize(600, 200);
        console.log('✓ Frame created:', frame.id);
        
        // Add some visualization based on the data
        let group: RectangleNode[] = [];
        if (data && data.normalized && data.normalized.length > 0) {
            console.log('Adding visualization elements...');
            const barWidth = 600 / data.normalized.length;
            let largestBar = 0;
            for (let i = 0; i < data.normalized.length; i++) {
                const bar = figma.createRectangle();
                const height = data.normalized[i] * 180;
                if (height > largestBar) {
                    largestBar = height
                }
                bar.resize(barWidth, height);
                bar.x = i * barWidth;
                bar.y = (frame.height / 2) - (bar.height / 2);
                bar.fills = [{ type: 'SOLID', color: { r: 1, g: 0, b: 0 } }];
                group.push(bar);
                frame.appendChild(bar);
            }
            if (data.resultPreference == "union") {
                const union = figma.union(group, frame);
                union.fills = [{ type: 'SOLID', color: { r: 1, g: 0, b: 0 } }]
                union.y = 0    
            }
            if (data.resultPreference == "group") {
                figma.group(group, frame);
            }
            frame.resize(frame.width, largestBar);
            console.log(`✓ Created ${data.normalized.length} bars`);
        }
        
        console.log('✓ Visualization complete');
    } catch (error) {
        console.error('✗ Error in buildEnergyViz:', error);
        throw error;
    }
}
  
// Shared shape drawing library for sprite generators
// All shapes are normalized to a 128x128 canvas with center at 64,64
// Use ctx.translate() and ctx.scale() to position and size shapes

const Shapes = {
    // ============================================
    // FILL SHAPES (used by globe generator)
    // ============================================
    
    circle: (ctx, percent, emptyColor, fillColor) => {
        const center = 64;
        const size = 128;
        
        ctx.fillStyle = emptyColor;
        ctx.beginPath();
        ctx.arc(center, center, size * 0.4, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = fillColor;
        ctx.beginPath();
        ctx.arc(center, center, size * 0.4 * percent, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.arc(center, center, size * 0.4, 0, Math.PI * 2);
        ctx.stroke();
    },

    bar: (ctx, percent, emptyColor, fillColor) => {
        const size = 128;
        const barHeight = size * 0.3;
        const barY = (size - barHeight) / 2;
        const barPadding = size * 0.1;
        const barWidth = size - barPadding * 2;
        
        ctx.fillStyle = emptyColor;
        ctx.fillRect(barPadding, barY, barWidth, barHeight);
        
        ctx.fillStyle = fillColor;
        ctx.fillRect(barPadding, barY, barWidth * percent, barHeight);
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        ctx.strokeRect(barPadding, barY, barWidth, barHeight);
    },

    vertical: (ctx, percent, emptyColor, fillColor) => {
        const size = 128;
        const vBarWidth = size * 0.3;
        const vBarX = (size - vBarWidth) / 2;
        const vBarPadding = size * 0.1;
        const vBarHeight = size - vBarPadding * 2;
        
        ctx.fillStyle = emptyColor;
        ctx.fillRect(vBarX, vBarPadding, vBarWidth, vBarHeight);
        
        const fillHeight = vBarHeight * percent;
        const fillY = vBarPadding + vBarHeight - fillHeight;
        ctx.fillStyle = fillColor;
        ctx.fillRect(vBarX, fillY, vBarWidth, fillHeight);
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        ctx.strokeRect(vBarX, vBarPadding, vBarWidth, vBarHeight);
    },

    radial: (ctx, percent, emptyColor, fillColor) => {
        const center = 64;
        const size = 128;
        const radius = size * 0.4;
        
        ctx.fillStyle = emptyColor;
        ctx.beginPath();
        ctx.arc(center, center, radius, 0, Math.PI * 2);
        ctx.fill();
        
        if (percent > 0) {
            ctx.fillStyle = fillColor;
            ctx.beginPath();
            ctx.moveTo(center, center);
            ctx.arc(center, center, radius, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * percent);
            ctx.closePath();
            ctx.fill();
        }
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.arc(center, center, radius, 0, Math.PI * 2);
        ctx.stroke();
    },

    heart: (ctx, percent, emptyColor, fillColor) => {
        const center = 64;
        const size = 128;
        
        ctx.save();
        ctx.translate(center, center - 5); // Shift up by 5 pixels to center better
        ctx.scale(size / 100, size / 100);
        
        const drawHeartPath = (context) => {
            context.beginPath();
            context.moveTo(0, 10);
            context.bezierCurveTo(-25, -15, -50, -5, -25, 25);
            context.lineTo(0, 45);
            context.lineTo(25, 25);
            context.bezierCurveTo(50, -5, 25, -15, 0, 10);
            context.closePath();
        };
        
        ctx.fillStyle = emptyColor;
        drawHeartPath(ctx);
        ctx.fill();
        
        // Fill from bottom to top (correct direction)
        ctx.save();
        ctx.beginPath();
        const clipHeight = 100 * percent;
        const clipY = 50 - clipHeight; // Start from bottom
        ctx.rect(-50, clipY, 100, clipHeight);
        ctx.clip();
        ctx.fillStyle = fillColor;
        drawHeartPath(ctx);
        ctx.fill();
        ctx.restore();
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        drawHeartPath(ctx);
        ctx.stroke();
        ctx.restore();
    },

    potion: (ctx, percent, emptyColor, fillColor) => {
        const size = 128;
        const center = 64;
        const bottleWidth = size * 0.5;
        const bottleHeight = size * 0.7;
        const bottleX = (size - bottleWidth) / 2;
        const bottleY = size * 0.15;
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        ctx.fillStyle = emptyColor;
        
        ctx.beginPath();
        ctx.moveTo(bottleX + bottleWidth * 0.2, bottleY);
        ctx.lineTo(bottleX + bottleWidth * 0.8, bottleY);
        ctx.lineTo(bottleX + bottleWidth, bottleY + bottleHeight * 0.2);
        ctx.lineTo(bottleX + bottleWidth, bottleY + bottleHeight);
        ctx.lineTo(bottleX, bottleY + bottleHeight);
        ctx.lineTo(bottleX, bottleY + bottleHeight * 0.2);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
        
        if (percent > 0) {
            const liquidHeight = bottleHeight * 0.8 * percent;
            const liquidY = bottleY + bottleHeight - liquidHeight;
            ctx.fillStyle = fillColor;
            ctx.fillRect(bottleX + 2, liquidY, bottleWidth - 4, liquidHeight - 2);
        }
        
        ctx.fillStyle = 'rgba(100,100,100,0.5)';
        ctx.fillRect(bottleX + bottleWidth * 0.35, bottleY - size * 0.1, bottleWidth * 0.3, size * 0.1);
        ctx.strokeRect(bottleX + bottleWidth * 0.35, bottleY - size * 0.1, bottleWidth * 0.3, size * 0.1);
    },

    orb: (ctx, percent, emptyColor, fillColor) => {
        const center = 64;
        const size = 128;
        
        let gradientColor = fillColor;
        let gradientColorSemi = fillColor;
        let gradientColorTransparent = fillColor;
        
        if (fillColor.startsWith('rgba')) {
            gradientColor = fillColor;
            const match = fillColor.match(/rgba\((\d+),\s*(\d+),\s*(\d+),\s*([\d.]+)\)/);
            if (match) {
                const [, r, g, b, a] = match;
                gradientColorSemi = `rgba(${r}, ${g}, ${b}, ${parseFloat(a) * 0.5})`;
                gradientColorTransparent = `rgba(${r}, ${g}, ${b}, 0)`;
            }
        } else {
            gradientColor = fillColor;
            gradientColorSemi = fillColor + '88';
            gradientColorTransparent = fillColor + '00';
        }
        
        const gradient = ctx.createRadialGradient(center, center, 0, center, center, size * 0.5);
        gradient.addColorStop(0, gradientColor);
        gradient.addColorStop(0.7, gradientColorSemi);
        gradient.addColorStop(1, gradientColorTransparent);
        
        ctx.fillStyle = emptyColor;
        ctx.beginPath();
        ctx.arc(center, center, size * 0.4, 0, Math.PI * 2);
        ctx.fill();
        
        if (percent > 0) {
            ctx.fillStyle = gradient;
            ctx.globalAlpha = percent;
            ctx.beginPath();
            ctx.arc(center, center, size * 0.5, 0, Math.PI * 2);
            ctx.fill();
            ctx.globalAlpha = 1;
        }
        
        ctx.fillStyle = fillColor;
        ctx.beginPath();
        ctx.arc(center, center, size * 0.4 * percent, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = 'rgba(255,255,255,0.4)';
        ctx.beginPath();
        ctx.arc(center - size * 0.1, center - size * 0.1, size * 0.15 * percent, 0, Math.PI * 2);
        ctx.fill();
    },

    diamond: (ctx, percent, emptyColor, fillColor) => {
        const center = 64;
        const size = 128;
        const dSize = size * 0.4;
        
        ctx.fillStyle = emptyColor;
        ctx.beginPath();
        ctx.moveTo(center, center - dSize);
        ctx.lineTo(center + dSize, center);
        ctx.lineTo(center, center + dSize);
        ctx.lineTo(center - dSize, center);
        ctx.closePath();
        ctx.fill();
        
        if (percent > 0) {
            ctx.save();
            ctx.beginPath();
            const clipHeight = dSize * 2 * percent;
            const clipY = center + dSize - clipHeight;
            ctx.rect(center - dSize, clipY, dSize * 2, clipHeight);
            ctx.clip();
            
            ctx.fillStyle = fillColor;
            ctx.beginPath();
            ctx.moveTo(center, center - dSize);
            ctx.lineTo(center + dSize, center);
            ctx.lineTo(center, center + dSize);
            ctx.lineTo(center - dSize, center);
            ctx.closePath();
            ctx.fill();
            ctx.restore();
        }
        
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(center, center - dSize);
        ctx.lineTo(center + dSize, center);
        ctx.lineTo(center, center + dSize);
        ctx.lineTo(center - dSize, center);
        ctx.closePath();
        ctx.stroke();
    },

    // ============================================
    // STATIC ICON SHAPES (no percent parameter)
    // ============================================
    
    fire: (ctx) => {
        ctx.fillStyle = '#ff6b00';
        ctx.beginPath();
        ctx.moveTo(64, 20);
        ctx.bezierCurveTo(80, 35, 85, 60, 85, 75);
        ctx.bezierCurveTo(85, 90, 76, 100, 64, 105);
        ctx.bezierCurveTo(52, 100, 43, 90, 43, 75);
        ctx.bezierCurveTo(43, 60, 48, 35, 64, 20);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#ffaa00';
        ctx.beginPath();
        ctx.moveTo(64, 30);
        ctx.bezierCurveTo(74, 42, 77, 58, 77, 70);
        ctx.bezierCurveTo(77, 82, 71, 90, 64, 93);
        ctx.bezierCurveTo(57, 90, 51, 82, 51, 70);
        ctx.bezierCurveTo(51, 58, 54, 42, 64, 30);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#ffeb3b';
        ctx.beginPath();
        ctx.moveTo(64, 40);
        ctx.bezierCurveTo(69, 48, 70, 58, 70, 66);
        ctx.bezierCurveTo(70, 74, 67, 80, 64, 82);
        ctx.bezierCurveTo(61, 80, 58, 74, 58, 66);
        ctx.bezierCurveTo(58, 58, 59, 48, 64, 40);
        ctx.closePath();
        ctx.fill();
    },

    ice: (ctx) => {
        ctx.fillStyle = '#4dd0e1';
        ctx.beginPath();
        ctx.moveTo(64, 25);
        ctx.lineTo(85, 50);
        ctx.lineTo(75, 85);
        ctx.lineTo(64, 95);
        ctx.lineTo(53, 85);
        ctx.lineTo(43, 50);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#b3e5fc';
        ctx.beginPath();
        ctx.moveTo(64, 25);
        ctx.lineTo(75, 50);
        ctx.lineTo(64, 60);
        ctx.lineTo(53, 50);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#80deea';
        ctx.beginPath();
        ctx.moveTo(64, 60);
        ctx.lineTo(75, 50);
        ctx.lineTo(75, 85);
        ctx.lineTo(64, 95);
        ctx.closePath();
        ctx.fill();
        
        ctx.beginPath();
        ctx.moveTo(64, 60);
        ctx.lineTo(53, 50);
        ctx.lineTo(53, 85);
        ctx.lineTo(64, 95);
        ctx.closePath();
        ctx.fill();
    },

    lightning: (ctx) => {
        ctx.fillStyle = '#ffeb3b';
        ctx.beginPath();
        ctx.moveTo(64, 20);
        ctx.lineTo(50, 60);
        ctx.lineTo(70, 60);
        ctx.lineTo(55, 108);
        ctx.lineTo(78, 68);
        ctx.lineTo(60, 68);
        ctx.lineTo(75, 20);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#fff59d';
        ctx.beginPath();
        ctx.moveTo(67, 25);
        ctx.lineTo(58, 60);
        ctx.lineTo(68, 60);
        ctx.lineTo(62, 80);
        ctx.lineTo(72, 68);
        ctx.lineTo(65, 68);
        ctx.lineTo(71, 25);
        ctx.closePath();
        ctx.fill();
    },

    poison: (ctx) => {
        ctx.fillStyle = '#66bb6a';
        ctx.beginPath();
        ctx.arc(64, 55, 30, Math.PI, 0, false);
        ctx.lineTo(64, 95);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#388e3c';
        ctx.beginPath();
        ctx.arc(64, 55, 20, Math.PI, 0, false);
        ctx.lineTo(64, 85);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#a5d6a7';
        ctx.beginPath();
        ctx.arc(64, 55, 10, Math.PI, 0, false);
        ctx.lineTo(64, 70);
        ctx.closePath();
        ctx.fill();
    },

    magic: (ctx) => {
        ctx.fillStyle = '#f06292';
        ctx.beginPath();
        
        for (let i = 0; i < 5; i++) {
            const angle = (Math.PI * 2 / 5) * i - Math.PI / 2;
            const x = 64 + Math.cos(angle) * 45;
            const y = 64 + Math.sin(angle) * 45;
            
            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
            
            const innerAngle = angle + (Math.PI / 5);
            const innerX = 64 + Math.cos(innerAngle) * 18;
            const innerY = 64 + Math.sin(innerAngle) * 18;
            ctx.lineTo(innerX, innerY);
        }
        
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#f8bbd0';
        ctx.beginPath();
        
        for (let i = 0; i < 5; i++) {
            const angle = (Math.PI * 2 / 5) * i - Math.PI / 2;
            const x = 64 + Math.cos(angle) * 25;
            const y = 64 + Math.sin(angle) * 25;
            
            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
            
            const innerAngle = angle + (Math.PI / 5);
            const innerX = 64 + Math.cos(innerAngle) * 10;
            const innerY = 64 + Math.sin(innerAngle) * 10;
            ctx.lineTo(innerX, innerY);
        }
        
        ctx.closePath();
        ctx.fill();
    },

    physical: (ctx) => {
        ctx.fillStyle = '#9e9e9e';
        ctx.beginPath();
        ctx.moveTo(64, 20);
        ctx.lineTo(70, 75);
        ctx.lineTo(64, 80);
        ctx.lineTo(58, 75);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#e0e0e0';
        ctx.beginPath();
        ctx.moveTo(64, 20);
        ctx.lineTo(66, 75);
        ctx.lineTo(64, 78);
        ctx.lineTo(62, 75);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#6d4c41';
        ctx.fillRect(45, 75, 38, 6);
        
        ctx.fillStyle = '#8d6e63';
        ctx.fillRect(58, 81, 12, 18);
        
        ctx.strokeStyle = '#6d4c41';
        ctx.lineWidth = 1.5;
        for (let i = 0; i < 4; i++) {
            ctx.beginPath();
            ctx.moveTo(58, 84 + i * 4);
            ctx.lineTo(70, 84 + i * 4);
            ctx.stroke();
        }
        
        ctx.fillStyle = '#6d4c41';
        ctx.beginPath();
        ctx.arc(64, 102, 5, 0, Math.PI * 2);
        ctx.fill();
    },

    sfx: (ctx) => {
        ctx.fillStyle = '#ffa726';
        ctx.beginPath();
        ctx.moveTo(45, 50);
        ctx.lineTo(45, 78);
        ctx.lineTo(60, 70);
        ctx.lineTo(60, 58);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillRect(35, 54, 10, 20);
        
        ctx.strokeStyle = '#ffb74d';
        ctx.lineWidth = 4;
        ctx.lineCap = 'round';
        
        ctx.beginPath();
        ctx.arc(60, 64, 12, -Math.PI/4, Math.PI/4, false);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.arc(60, 64, 22, -Math.PI/4, Math.PI/4, false);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.arc(60, 64, 32, -Math.PI/4, Math.PI/4, false);
        ctx.stroke();
    },

    fx: (ctx) => {
        ctx.fillStyle = '#ab47bc';
        ctx.beginPath();
        ctx.arc(64, 64, 12, 0, Math.PI * 2);
        ctx.fill();
        
        const rays = 8;
        for (let i = 0; i < rays; i++) {
            const angle = (Math.PI * 2 / rays) * i;
            const x1 = 64 + Math.cos(angle) * 15;
            const y1 = 64 + Math.sin(angle) * 15;
            const x2 = 64 + Math.cos(angle) * 40;
            const y2 = 64 + Math.sin(angle) * 40;
            
            const angleLeft = angle - 0.3;
            const angleRight = angle + 0.3;
            const x3 = 64 + Math.cos(angleLeft) * 35;
            const y3 = 64 + Math.sin(angleLeft) * 35;
            const x4 = 64 + Math.cos(angleRight) * 35;
            const y4 = 64 + Math.sin(angleRight) * 35;
            
            ctx.beginPath();
            ctx.moveTo(x1, y1);
            ctx.lineTo(x3, y3);
            ctx.lineTo(x2, y2);
            ctx.lineTo(x4, y4);
            ctx.closePath();
            ctx.fill();
        }
        
        ctx.fillStyle = '#ce93d8';
        ctx.beginPath();
        ctx.arc(64, 64, 8, 0, Math.PI * 2);
        ctx.fill();
    },

    // Additional icons
    water: (ctx) => {
        ctx.fillStyle = '#2196f3';
        ctx.beginPath();
        ctx.moveTo(64, 20);
        ctx.bezierCurveTo(40, 40, 30, 60, 30, 75);
        ctx.bezierCurveTo(30, 95, 45, 108, 64, 108);
        ctx.bezierCurveTo(83, 108, 98, 95, 98, 75);
        ctx.bezierCurveTo(98, 60, 88, 40, 64, 20);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#64b5f6';
        ctx.beginPath();
        ctx.arc(50, 65, 8, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = '#90caf9';
        ctx.beginPath();
        ctx.arc(75, 70, 6, 0, Math.PI * 2);
        ctx.fill();
    },

    earth: (ctx) => {
        ctx.fillStyle = '#8d6e63';
        ctx.beginPath();
        ctx.moveTo(64, 30);
        ctx.lineTo(50, 50);
        ctx.lineTo(50, 80);
        ctx.lineTo(64, 95);
        ctx.lineTo(78, 80);
        ctx.lineTo(78, 50);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#a1887f';
        for (let i = 0; i < 5; i++) {
            const y = 45 + i * 10;
            ctx.fillRect(55, y, 18, 6);
        }
        
        ctx.fillStyle = '#6d4c41';
        ctx.beginPath();
        ctx.arc(64, 35, 5, 0, Math.PI * 2);
        ctx.fill();
    },

    wind: (ctx) => {
        ctx.strokeStyle = '#b2ebf2';
        ctx.lineWidth = 6;
        ctx.lineCap = 'round';
        
        // Top curve
        ctx.beginPath();
        ctx.moveTo(25, 35);
        ctx.bezierCurveTo(45, 25, 70, 35, 90, 30);
        ctx.bezierCurveTo(95, 28, 95, 35, 90, 37);
        ctx.stroke();
        
        // Middle curve
        ctx.beginPath();
        ctx.moveTo(30, 55);
        ctx.bezierCurveTo(50, 48, 75, 55, 100, 52);
        ctx.bezierCurveTo(105, 51, 105, 57, 100, 58);
        ctx.stroke();
        
        // Bottom curve
        ctx.beginPath();
        ctx.moveTo(20, 75);
        ctx.bezierCurveTo(40, 68, 65, 75, 85, 72);
        ctx.bezierCurveTo(90, 71, 90, 77, 85, 78);
        ctx.stroke();
        
        // Smaller accent curves
        ctx.strokeStyle = '#e0f7fa';
        ctx.lineWidth = 4;
        
        ctx.beginPath();
        ctx.moveTo(40, 45);
        ctx.bezierCurveTo(50, 42, 60, 45, 70, 43);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.moveTo(45, 65);
        ctx.bezierCurveTo(55, 62, 65, 65, 75, 63);
        ctx.stroke();
    },

    dark: (ctx) => {
        ctx.fillStyle = '#424242';
        ctx.beginPath();
        ctx.arc(64, 64, 40, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = '#212121';
        ctx.beginPath();
        ctx.moveTo(64, 24);
        for (let i = 0; i < 8; i++) {
            const angle = (Math.PI * 2 / 8) * i - Math.PI / 2;
            const outerR = i % 2 === 0 ? 50 : 40;
            const x = 64 + Math.cos(angle) * outerR;
            const y = 64 + Math.sin(angle) * outerR;
            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        }
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#9e9e9e';
        ctx.beginPath();
        ctx.arc(64, 64, 15, 0, Math.PI * 2);
        ctx.fill();
    },

    light: (ctx) => {
        const gradient = ctx.createRadialGradient(64, 64, 0, 64, 64, 50);
        gradient.addColorStop(0, '#fff9c4');
        gradient.addColorStop(0.5, '#ffeb3b');
        gradient.addColorStop(1, '#fdd835');
        
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(64, 64, 25, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = '#ffeb3b';
        for (let i = 0; i < 12; i++) {
            const angle = (Math.PI * 2 / 12) * i;
            ctx.save();
            ctx.translate(64, 64);
            ctx.rotate(angle);
            ctx.beginPath();
            ctx.moveTo(0, -30);
            ctx.lineTo(-4, -45);
            ctx.lineTo(0, -50);
            ctx.lineTo(4, -45);
            ctx.closePath();
            ctx.fill();
            ctx.restore();
        }
        
        ctx.fillStyle = '#fffde7';
        ctx.beginPath();
        ctx.arc(64, 64, 15, 0, Math.PI * 2);
        ctx.fill();
    },

    holy: (ctx) => {
        ctx.fillStyle = '#ffe082';
        ctx.beginPath();
        ctx.arc(64, 45, 20, 0, Math.PI * 2);
        ctx.fill();
        
        const gradient = ctx.createRadialGradient(64, 45, 0, 64, 45, 30);
        gradient.addColorStop(0, 'rgba(255, 255, 255, 0.8)');
        gradient.addColorStop(1, 'rgba(255, 224, 130, 0)');
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(64, 45, 30, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = '#ffb74d';
        ctx.fillRect(60, 65, 8, 30);
        ctx.fillRect(45, 75, 38, 8);
        
        ctx.fillStyle = '#ffa726';
        ctx.fillRect(61, 66, 6, 28);
        ctx.fillRect(46, 76, 36, 6);
    },

    curse: (ctx) => {
        ctx.fillStyle = '#7e57c2';
        
        for (let i = 0; i < 6; i++) {
            const angle = (Math.PI * 2 / 6) * i - Math.PI / 2;
            const x = 64 + Math.cos(angle) * 35;
            const y = 64 + Math.sin(angle) * 35;
            
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(angle + Math.PI / 2);
            ctx.beginPath();
            ctx.moveTo(0, -8);
            ctx.lineTo(6, 0);
            ctx.lineTo(0, 12);
            ctx.lineTo(-6, 0);
            ctx.closePath();
            ctx.fill();
            ctx.restore();
        }
        
        ctx.fillStyle = '#9575cd';
        ctx.beginPath();
        ctx.arc(64, 64, 18, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.fillStyle = '#4a148c';
        ctx.font = 'bold 24px serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('☠', 64, 64);
    },

    healing: (ctx) => {
        ctx.fillStyle = '#4caf50';
        ctx.fillRect(54, 34, 20, 60);
        ctx.fillRect(34, 54, 60, 20);
        
        ctx.fillStyle = '#66bb6a';
        ctx.fillRect(56, 36, 16, 56);
        ctx.fillRect(36, 56, 56, 16);
        
        ctx.fillStyle = '#a5d6a7';
        ctx.fillRect(58, 38, 12, 52);
        ctx.fillRect(38, 58, 52, 12);
    },

    shield: (ctx) => {
        ctx.fillStyle = '#546e7a';
        ctx.beginPath();
        ctx.moveTo(64, 20);
        ctx.lineTo(95, 35);
        ctx.lineTo(95, 70);
        ctx.bezierCurveTo(95, 85, 80, 100, 64, 108);
        ctx.bezierCurveTo(48, 100, 33, 85, 33, 70);
        ctx.lineTo(33, 35);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#78909c';
        ctx.beginPath();
        ctx.moveTo(64, 25);
        ctx.lineTo(90, 38);
        ctx.lineTo(90, 70);
        ctx.bezierCurveTo(90, 83, 78, 95, 64, 102);
        ctx.lineTo(64, 25);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#cfd8dc';
        ctx.beginPath();
        ctx.arc(64, 55, 12, 0, Math.PI * 2);
        ctx.fill();
    },

    attack: (ctx) => {
        ctx.fillStyle = '#f44336';
        ctx.beginPath();
        ctx.moveTo(35, 64);
        ctx.lineTo(55, 44);
        ctx.lineTo(75, 44);
        ctx.lineTo(95, 64);
        ctx.lineTo(75, 84);
        ctx.lineTo(55, 84);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#e57373';
        ctx.beginPath();
        ctx.moveTo(45, 64);
        ctx.lineTo(55, 54);
        ctx.lineTo(75, 54);
        ctx.lineTo(85, 64);
        ctx.lineTo(75, 74);
        ctx.lineTo(55, 74);
        ctx.closePath();
        ctx.fill();
        
        ctx.fillStyle = '#ffebee';
        ctx.beginPath();
        ctx.moveTo(55, 64);
        ctx.lineTo(60, 59);
        ctx.lineTo(70, 59);
        ctx.lineTo(75, 64);
        ctx.lineTo(70, 69);
        ctx.lineTo(60, 69);
        ctx.closePath();
        ctx.fill();
    },

    defense: (ctx) => {
        ctx.fillStyle = '#1976d2';
        ctx.strokeStyle = '#1565c0';
        ctx.lineWidth = 3;
        
        ctx.beginPath();
        ctx.arc(64, 64, 35, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();
        
        ctx.strokeStyle = '#42a5f5';
        ctx.lineWidth = 8;
        ctx.beginPath();
        ctx.arc(64, 64, 28, -Math.PI * 0.3, Math.PI * 0.3);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.arc(64, 64, 28, Math.PI * 0.4, Math.PI * 0.9);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.arc(64, 64, 28, Math.PI * 1.1, Math.PI * 1.6);
        ctx.stroke();
    },

    speed: (ctx) => {
        ctx.fillStyle = '#00bcd4';
        
        for (let i = 0; i < 4; i++) {
            const x = 30 + i * 15;
            const y = 64 + (i % 2 === 0 ? -10 : 10);
            
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(-0.3);
            
            ctx.beginPath();
            ctx.moveTo(-5, -8);
            ctx.lineTo(20, 0);
            ctx.lineTo(-5, 8);
            ctx.closePath();
            ctx.fill();
            
            ctx.restore();
        }
        
        ctx.fillStyle = '#26c6da';
        ctx.save();
        ctx.translate(85, 64);
        ctx.scale(1.5, 1.5);
        ctx.beginPath();
        ctx.moveTo(-5, -8);
        ctx.lineTo(20, 0);
        ctx.lineTo(-5, 8);
        ctx.closePath();
        ctx.fill();
        ctx.restore();
    },

    time: (ctx) => {
        ctx.fillStyle = '#9c27b0';
        ctx.beginPath();
        ctx.arc(64, 64, 35, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.strokeStyle = '#ce93d8';
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.arc(64, 64, 35, 0, Math.PI * 2);
        ctx.stroke();
        
        for (let i = 0; i < 12; i++) {
            const angle = (Math.PI * 2 / 12) * i - Math.PI / 2;
            const x1 = 64 + Math.cos(angle) * 28;
            const y1 = 64 + Math.sin(angle) * 28;
            const x2 = 64 + Math.cos(angle) * 32;
            const y2 = 64 + Math.sin(angle) * 32;
            
            ctx.strokeStyle = i % 3 === 0 ? '#e1bee7' : '#ba68c8';
            ctx.lineWidth = i % 3 === 0 ? 3 : 2;
            ctx.beginPath();
            ctx.moveTo(x1, y1);
            ctx.lineTo(x2, y2);
            ctx.stroke();
        }
        
        ctx.strokeStyle = '#e1bee7';
        ctx.lineWidth = 4;
        ctx.lineCap = 'round';
        ctx.beginPath();
        ctx.moveTo(64, 64);
        ctx.lineTo(64, 40);
        ctx.stroke();
        
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.moveTo(64, 64);
        ctx.lineTo(80, 64);
        ctx.stroke();
        
        ctx.fillStyle = '#f3e5f5';
        ctx.beginPath();
        ctx.arc(64, 64, 4, 0, Math.PI * 2);
        ctx.fill();
    }
};

// Metadata for all icons
Shapes.metadata = {
    // Fill shapes (used by globe generator)
    circle: { name: 'Circle', type: 'fill', color: '#666666' },
    bar: { name: 'Bar', type: 'fill', color: '#666666' },
    vertical: { name: 'Vertical', type: 'fill', color: '#666666' },
    radial: { name: 'Radial', type: 'fill', color: '#666666' },
    heart: { name: 'Heart', type: 'fill', color: '#e91e63' },
    potion: { name: 'Potion', type: 'fill', color: '#9c27b0' },
    orb: { name: 'Orb', type: 'fill', color: '#2196f3' },
    diamond: { name: 'Diamond', type: 'fill', color: '#00bcd4' },
    
    // Static icons
    fire: { name: 'Fire', type: 'icon', color: '#ff4500' },
    ice: { name: 'Ice', type: 'icon', color: '#00bfff' },
    lightning: { name: 'Lightning', type: 'icon', color: '#ffd700' },
    poison: { name: 'Poison', type: 'icon', color: '#4caf50' },
    magic: { name: 'Magic', type: 'icon', color: '#e91e63' },
    physical: { name: 'Physical', type: 'icon', color: '#795548' },
    sfx: { name: 'SFX', type: 'icon', color: '#ff9800' },
    fx: { name: 'FX', type: 'icon', color: '#9c27b0' },
    water: { name: 'Water', type: 'icon', color: '#2196f3' },
    earth: { name: 'Earth', type: 'icon', color: '#8d6e63' },
    wind: { name: 'Wind', type: 'icon', color: '#b2ebf2' },
    dark: { name: 'Dark', type: 'icon', color: '#424242' },
    light: { name: 'Light', type: 'icon', color: '#ffeb3b' },
    holy: { name: 'Holy', type: 'icon', color: '#ffe082' },
    curse: { name: 'Curse', type: 'icon', color: '#7e57c2' },
    healing: { name: 'Healing', type: 'icon', color: '#4caf50' },
    shield: { name: 'Shield', type: 'icon', color: '#546e7a' },
    attack: { name: 'Attack', type: 'icon', color: '#f44336' },
    defense: { name: 'Defense', type: 'icon', color: '#1976d2' },
    speed: { name: 'Speed', type: 'icon', color: '#00bcd4' },
    time: { name: 'Time', type: 'icon', color: '#9c27b0' }
};

// Helper to get all icons (excludes fill shapes)
Shapes.getIcons = function() {
    return Object.keys(this.metadata)
        .filter(key => this.metadata[key].type === 'icon')
        .map(key => ({
            id: key,
            name: this.metadata[key].name,
            color: this.metadata[key].color,
            draw: this[key]
        }));
};

// Helper to get all fill shapes
Shapes.getFillShapes = function() {
    return Object.keys(this.metadata)
        .filter(key => this.metadata[key].type === 'fill')
        .map(key => ({
            id: key,
            name: this.metadata[key].name,
            draw: this[key]
        }));
};

// Add section-based drawing to the circle shape
Shapes.circleSections = function(ctx, percent, emptyColor, fillColor, sectionCount, fillStyle) {
    const center = 64;
    const size = 128;
    const radius = size * 0.4;
    
    // Draw empty circle
    ctx.fillStyle = emptyColor;
    ctx.beginPath();
    ctx.arc(center, center, radius, 0, Math.PI * 2);
    ctx.fill();
    
    // Calculate how many sections should be filled
    const totalSections = sectionCount;
    let filledSections = 0;
    let partialFill = 0;
    
    if (fillStyle === 'sequential') {
        filledSections = Math.floor(percent * totalSections);
        partialFill = (percent * totalSections) - filledSections;
    } else { // binary
        filledSections = Math.round(percent * totalSections);
        partialFill = 0;
    }
    
    // Draw filled sections
    if (filledSections > 0 || partialFill > 0) {
        ctx.fillStyle = fillColor;
        
        const sectionAngle = (Math.PI * 2) / totalSections;
        
        // Draw completely filled sections
        for (let i = 0; i < filledSections; i++) {
            const startAngle = -Math.PI/2 + (i * sectionAngle);
            const endAngle = startAngle + sectionAngle;
            
            ctx.beginPath();
            ctx.moveTo(center, center);
            ctx.arc(center, center, radius, startAngle, endAngle);
            ctx.closePath();
            ctx.fill();
        }
        
        // Draw partially filled section
        if (partialFill > 0 && filledSections < totalSections) {
            const startAngle = -Math.PI/2 + (filledSections * sectionAngle);
            const endAngle = startAngle + (sectionAngle * partialFill);
            
            ctx.beginPath();
            ctx.moveTo(center, center);
            ctx.arc(center, center, radius, startAngle, endAngle);
            ctx.closePath();
            ctx.fill();
        }
    }
    
    // Draw section dividers
    ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(center, center, radius, 0, Math.PI * 2);
    ctx.stroke();
    
    // Draw section lines
    ctx.strokeStyle = 'rgba(255,255,255,0.2)';
    ctx.lineWidth = 1;
    for (let i = 0; i < totalSections; i++) {
        const angle = -Math.PI/2 + (i * (Math.PI * 2 / totalSections));
        const x = center + Math.cos(angle) * radius;
        const y = center + Math.sin(angle) * radius;
        
        ctx.beginPath();
        ctx.moveTo(center, center);
        ctx.lineTo(x, y);
        ctx.stroke();
    }
};

// Similarly, we can create section versions for other shapes
Shapes.barSections = function(ctx, percent, emptyColor, fillColor, sectionCount, fillStyle) {
    const size = 128;
    const barHeight = size * 0.3;
    const barY = (size - barHeight) / 2;
    const barPadding = size * 0.1;
    const barWidth = size - barPadding * 2;
    const sectionWidth = barWidth / sectionCount;
    
    // Draw empty bar
    ctx.fillStyle = emptyColor;
    ctx.fillRect(barPadding, barY, barWidth, barHeight);
    
    // Calculate filled sections
    const totalSections = sectionCount;
    let filledSections = 0;
    let partialFill = 0;
    
    if (fillStyle === 'sequential') {
        filledSections = Math.floor(percent * totalSections);
        partialFill = (percent * totalSections) - filledSections;
    } else { // binary
        filledSections = Math.round(percent * totalSections);
        partialFill = 0;
    }
    
    // Draw filled sections
    if (filledSections > 0 || partialFill > 0) {
        ctx.fillStyle = fillColor;
        
        // Draw completely filled sections
        for (let i = 0; i < filledSections; i++) {
            const x = barPadding + (i * sectionWidth);
            ctx.fillRect(x, barY, sectionWidth, barHeight);
        }
        
        // Draw partially filled section
        if (partialFill > 0 && filledSections < totalSections) {
            const x = barPadding + (filledSections * sectionWidth);
            ctx.fillRect(x, barY, sectionWidth * partialFill, barHeight);
        }
    }
    
    // Draw section dividers
    ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    ctx.lineWidth = 2;
    ctx.strokeRect(barPadding, barY, barWidth, barHeight);
    
    // Draw section lines
    ctx.strokeStyle = 'rgba(255,255,255,0.2)';
    ctx.lineWidth = 1;
    for (let i = 1; i < sectionCount; i++) {
        const x = barPadding + (i * sectionWidth);
        ctx.beginPath();
        ctx.moveTo(x, barY);
        ctx.lineTo(x, barY + barHeight);
        ctx.stroke();
    }
};

// Export
if (typeof module !== 'undefined' && module.exports) {
    module.exports = Shapes;
}
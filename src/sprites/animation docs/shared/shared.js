/**
 * Shared Core Library for Sprite Animation Tools
 * Universal module that works in static HTML and served environments
 * 
 * Usage in HTML:
 * <script src="../shared/shared-core.js"></script>
 * <script>
 *   const { color, canvas, files } = SpriteTools;
 *   // Use utilities...
 * </script>
 */

(function(global) {
    'use strict';

    function debugFrames() {
        console.log('frames var:', typeof frames, frames ? frames.length : 'undefined');
        try {
            console.log('frameCount input value:', document.getElementById('frameCount')?.value);
            console.log('size input value:', document.getElementById('size')?.value);
            console.log('style value:', document.getElementById('style')?.value);
            console.log('colorScheme value:', document.getElementById('colorScheme')?.value);
        } catch(e) { console.warn('debug error', e); }
    }

    // ==============================================
    // COLOR UTILITIES
    // ==============================================
    const color = {
        hexToRgb(hex) {
            const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
            return result ? {
                r: parseInt(result[1], 16),
                g: parseInt(result[2], 16),
                b: parseInt(result[3], 16)
            } : null;
        },

        rgbToHex(r, g, b) {
            return "#" + [r, g, b].map(x => {
                const hex = Math.round(x).toString(16);
                return hex.length === 1 ? "0" + hex : hex;
            }).join('');
        },

        lerp(color1, color2, t) {
            if (color1 === 'transparent' && color2 === 'transparent') {
                return 'rgba(0,0,0,0)';
            }
            if (color1 === 'transparent') {
                const c2 = this.hexToRgb(color2);
                return `rgba(${c2.r}, ${c2.g}, ${c2.b}, ${t})`;
            }
            if (color2 === 'transparent') {
                const c1 = this.hexToRgb(color1);
                return `rgba(${c1.r}, ${c1.g}, ${c1.b}, ${1-t})`;
            }
            
            const c1 = this.hexToRgb(color1);
            const c2 = this.hexToRgb(color2);
            return this.rgbToHex(
                c1.r + (c2.r - c1.r) * t,
                c1.g + (c2.g - c1.g) * t,
                c1.b + (c2.b - c1.b) * t
            );
        },

        getGradientColor(percent, stops) {
            if (stops.length === 1) {
                return stops[0].color === 'transparent' ? 'rgba(0,0,0,0)' : stops[0].color;
            }
            
            const sorted = [...stops].sort((a, b) => a.position - b.position);
            
            for (let i = 0; i < sorted.length - 1; i++) {
                if (percent >= sorted[i].position && percent <= sorted[i + 1].position) {
                    const segmentSize = sorted[i + 1].position - sorted[i].position;
                    const t = (percent - sorted[i].position) / segmentSize;
                    return this.lerp(sorted[i].color, sorted[i + 1].color, t);
                }
            }
            
            if (percent <= sorted[0].position) {
                return sorted[0].color === 'transparent' ? 'rgba(0,0,0,0)' : sorted[0].color;
            }
            return sorted[sorted.length - 1].color === 'transparent' ? 'rgba(0,0,0,0)' : sorted[sorted.length - 1].color;
        }
    };

    // ==============================================
    // CANVAS UTILITIES
    // ==============================================
    const canvas = {
        create(width, height) {
            const canvas = document.createElement('canvas');
            canvas.width = width;
            canvas.height = height;
            const ctx = canvas.getContext('2d');
            return { canvas, ctx };
        },

        clear(ctx, x = 0, y = 0, w, h) {
            if (!w) w = ctx.canvas.width;
            if (!h) h = ctx.canvas.height;
            ctx.clearRect(x, y, w, h);
        },

        shapes: {
            circle(ctx, x, y, radius, color, fill = true) {
                ctx.beginPath();
                ctx.arc(x, y, radius, 0, Math.PI * 2);
                if (fill) {
                    ctx.fillStyle = color;
                    ctx.fill();
                } else {
                    ctx.strokeStyle = color;
                    ctx.stroke();
                }
            },

            rect(ctx, x, y, width, height, color, fill = true) {
                if (fill) {
                    ctx.fillStyle = color;
                    ctx.fillRect(x, y, width, height);
                } else {
                    ctx.strokeStyle = color;
                    ctx.strokeRect(x, y, width, height);
                }
            },

            roundRect(ctx, x, y, width, height, radius, color, fill = true) {
                ctx.beginPath();
                ctx.moveTo(x + radius, y);
                ctx.lineTo(x + width - radius, y);
                ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
                ctx.lineTo(x + width, y + height - radius);
                ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
                ctx.lineTo(x + radius, y + height);
                ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
                ctx.lineTo(x, y + radius);
                ctx.quadraticCurveTo(x, y, x + radius, y);
                ctx.closePath();
                
                if (fill) {
                    ctx.fillStyle = color;
                    ctx.fill();
                } else {
                    ctx.strokeStyle = color;
                    ctx.stroke();
                }
            },

            star(ctx, x, y, points, outerRadius, innerRadius, color, fill = true) {
                ctx.beginPath();
                for (let i = 0; i < points * 2; i++) {
                    const radius = i % 2 === 0 ? outerRadius : innerRadius;
                    const angle = (Math.PI / points) * i - Math.PI / 2;
                    const px = x + Math.cos(angle) * radius;
                    const py = y + Math.sin(angle) * radius;
                    
                    if (i === 0) {
                        ctx.moveTo(px, py);
                    } else {
                        ctx.lineTo(px, py);
                    }
                }
                ctx.closePath();
                
                if (fill) {
                    ctx.fillStyle = color;
                    ctx.fill();
                } else {
                    ctx.strokeStyle = color;
                    ctx.stroke();
                }
            }
        },

        gradients: {
            linear(ctx, x0, y0, x1, y1, stops) {
                const gradient = ctx.createLinearGradient(x0, y0, x1, y1);
                stops.forEach(stop => {
                    gradient.addColorStop(stop.position, stop.color);
                });
                return gradient;
            },

            radial(ctx, x, y, r0, r1, stops) {
                const gradient = ctx.createRadialGradient(x, y, r0, x, y, r1);
                stops.forEach(stop => {
                    gradient.addColorStop(stop.position, stop.color);
                });
                return gradient;
            }
        },

        effects: {
            glow(ctx, x, y, radius, color, blur = 10) {
                ctx.save();
                ctx.shadowColor = color;
                ctx.shadowBlur = blur;
                ctx.fillStyle = color;
                ctx.beginPath();
                ctx.arc(x, y, radius, 0, Math.PI * 2);
                ctx.fill();
                ctx.restore();
            }
        }
    };

    // ==============================================
    // FILE UTILITIES
    // ==============================================
    const files = {
        downloadCanvas(canvas, filename) {
            canvas.toBlob(blob => {
                this.downloadBlob(blob, filename);
            });
        },

        downloadBlob(blob, filename) {
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            a.click();
            URL.revokeObjectURL(url);
        },

        downloadJSON(obj, filename) {
            const jsonStr = JSON.stringify(obj, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            this.downloadBlob(blob, filename);
        },

        downloadText(text, filename) {
            const blob = new Blob([text], { type: 'text/plain' });
            this.downloadBlob(blob, filename);
        },

        loadImage(file) {
            return new Promise((resolve, reject) => {
                if (!file || !file.type.startsWith('image/')) {
                    reject(new Error('Invalid file type'));
                    return;
                }

                const reader = new FileReader();
                reader.onload = (e) => {
                    const img = new Image();
                    img.onload = () => resolve(img);
                    img.onerror = () => reject(new Error('Failed to load image'));
                    img.src = e.target.result;
                };
                reader.onerror = () => reject(new Error('Failed to read file'));
                reader.readAsDataURL(file);
            });
        },

        loadJSON(file) {
            return new Promise((resolve, reject) => {
                if (!file) {
                    reject(new Error('No file provided'));
                    return;
                }

                const reader = new FileReader();
                reader.onload = (e) => {
                    try {
                        const json = JSON.parse(e.target.result);
                        resolve(json);
                    } catch (err) {
                        reject(new Error('Invalid JSON: ' + err.message));
                    }
                };
                reader.onerror = () => reject(new Error('Failed to read file'));
                reader.readAsText(file);
            });
        },

        selectFile(accept = '*', multiple = false) {
            return new Promise((resolve) => {
                const input = document.createElement('input');
                input.type = 'file';
                input.accept = accept;
                input.multiple = multiple;
                
                input.onchange = (e) => {
                    const files = Array.from(e.target.files);
                    resolve(multiple ? files : files[0]);
                };
                
                input.click();
            });
        },

        async downloadCanvasesAsZip(canvases, zipFilename, filePrefix = '') {
            if (typeof JSZip === 'undefined') {
                throw new Error('JSZip library not loaded. Include: <script src="https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js"></script>');
            }

            const zip = new JSZip();
            const folder = zip.folder('frames');
            
            const blobPromises = canvases.map((canvas, i) => {
                return new Promise((resolve) => {
                    canvas.toBlob(blob => resolve({ index: i, blob }));
                });
            });

            const blobs = await Promise.all(blobPromises);
            
            blobs.forEach(({ index, blob }) => {
                folder.file(`${filePrefix}${index}.png`, blob);
            });

            const content = await zip.generateAsync({ type: 'blob' });
            this.downloadBlob(content, zipFilename);
        },

        createSpriteSheet(images, layout = 'horizontal', gridColumns = 4) {
            if (!images || images.length === 0) {
                throw new Error('No images provided');
            }

            const frameWidth = images[0].width;
            const frameHeight = images[0].height;
            const frameCount = images.length;

            let sheetWidth, sheetHeight;

            if (layout === 'horizontal') {
                sheetWidth = frameWidth * frameCount;
                sheetHeight = frameHeight;
            } else if (layout === 'vertical') {
                sheetWidth = frameWidth;
                sheetHeight = frameHeight * frameCount;
            } else if (layout === 'grid') {
                const rows = Math.ceil(frameCount / gridColumns);
                sheetWidth = frameWidth * gridColumns;
                sheetHeight = frameHeight * rows;
            }

            const { canvas, ctx } = canvas.create(sheetWidth, sheetHeight);

            images.forEach((img, i) => {
                let x, y;

                if (layout === 'horizontal') {
                    x = i * frameWidth;
                    y = 0;
                } else if (layout === 'vertical') {
                    x = 0;
                    y = i * frameHeight;
                } else if (layout === 'grid') {
                    const col = i % gridColumns;
                    const row = Math.floor(i / gridColumns);
                    x = col * frameWidth;
                    y = row * frameHeight;
                }

                ctx.drawImage(img, x, y);
            });

            return canvas;
        }
    };

    const frames = {
        // Auto-generate on control changes
        setupAutoGenerate: (generateFunction, controlSelectors) => {
            document.addEventListener('DOMContentLoaded', () => {
                const controls = document.querySelectorAll(controlSelectors);
                controls.forEach(control => {
                    control.addEventListener('change', generateFunction);
                    control.addEventListener('input', generateFunction);
                });
            });
        },
    
        // Create individual frame canvas
        createFrameCanvas: (width, height) => {
            const canvas = document.createElement('canvas');
            canvas.width = width;
            canvas.height = height;
            return canvas;
        },
    
        // Create frame display element with label
        createFrameDisplay: (canvas, label, onClick) => {
            const item = document.createElement('div');
            item.className = 'frame-item';
            item.onclick = onClick;
    
            const displayCanvas = document.createElement('canvas');
            displayCanvas.width = 64;
            displayCanvas.height = 64;
            const displayCtx = displayCanvas.getContext('2d');
            displayCtx.drawImage(canvas, 0, 0, 64, 64);
    
            const labelDiv = document.createElement('div');
            labelDiv.className = 'frame-label';
            labelDiv.textContent = label;
    
            item.appendChild(displayCanvas);
            item.appendChild(labelDiv);
            return item;
        }
    };
    // const shapes = {
    //     // Basic geometric shapes for globe generator
    //     circle: (ctx, percent, emptyColor, fillColor) => {
    //         const center = 64;
    //         const size = 128;
            
    //         ctx.fillStyle = emptyColor;
    //         ctx.beginPath();
    //         ctx.arc(center, center, size * 0.4, 0, Math.PI * 2);
    //         ctx.fill();
            
    //         ctx.fillStyle = fillColor;
    //         ctx.beginPath();
    //         ctx.arc(center, center, size * 0.4 * percent, 0, Math.PI * 2);
    //         ctx.fill();
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         ctx.beginPath();
    //         ctx.arc(center, center, size * 0.4, 0, Math.PI * 2);
    //         ctx.stroke();
    //     },
    
    //     bar: (ctx, percent, emptyColor, fillColor) => {
    //         const size = 128;
    //         const barHeight = size * 0.3;
    //         const barY = (size - barHeight) / 2;
    //         const barPadding = size * 0.1;
    //         const barWidth = size - barPadding * 2;
            
    //         ctx.fillStyle = emptyColor;
    //         ctx.fillRect(barPadding, barY, barWidth, barHeight);
            
    //         ctx.fillStyle = fillColor;
    //         ctx.fillRect(barPadding, barY, barWidth * percent, barHeight);
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         ctx.strokeRect(barPadding, barY, barWidth, barHeight);
    //     },
    
    //     vertical: (ctx, percent, emptyColor, fillColor) => {
    //         const size = 128;
    //         const vBarWidth = size * 0.3;
    //         const vBarX = (size - vBarWidth) / 2;
    //         const vBarPadding = size * 0.1;
    //         const vBarHeight = size - vBarPadding * 2;
            
    //         ctx.fillStyle = emptyColor;
    //         ctx.fillRect(vBarX, vBarPadding, vBarWidth, vBarHeight);
            
    //         const fillHeight = vBarHeight * percent;
    //         const fillY = vBarPadding + vBarHeight - fillHeight;
    //         ctx.fillStyle = fillColor;
    //         ctx.fillRect(vBarX, fillY, vBarWidth, fillHeight);
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         ctx.strokeRect(vBarX, vBarPadding, vBarWidth, vBarHeight);
    //     },
    
    //     radial: (ctx, percent, emptyColor, fillColor) => {
    //         const center = 64;
    //         const size = 128;
    //         const radius = size * 0.4;
            
    //         ctx.fillStyle = emptyColor;
    //         ctx.beginPath();
    //         ctx.arc(center, center, radius, 0, Math.PI * 2);
    //         ctx.fill();
            
    //         if (percent > 0) {
    //             ctx.fillStyle = fillColor;
    //             ctx.beginPath();
    //             ctx.moveTo(center, center);
    //             ctx.arc(center, center, radius, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * percent);
    //             ctx.closePath();
    //             ctx.fill();
    //         }
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         ctx.beginPath();
    //         ctx.arc(center, center, radius, 0, Math.PI * 2);
    //         ctx.stroke();
    //     },
    
    //     heart: (ctx, percent, emptyColor, fillColor) => {
    //         const center = 64;
    //         const size = 128;
            
    //         ctx.save();
    //         ctx.translate(center, center);
    //         ctx.scale(size / 100, size / 100);
            
    //         const drawHeartPath = (context) => {
    //             context.beginPath();
    //             context.moveTo(0, 15);
    //             context.bezierCurveTo(-25, -10, -50, 0, -25, 30);
    //             context.lineTo(0, 50);
    //             context.lineTo(25, 30);
    //             context.bezierCurveTo(50, 0, 25, -10, 0, 15);
    //             context.closePath();
    //         };
            
    //         ctx.fillStyle = emptyColor;
    //         drawHeartPath(ctx);
    //         ctx.fill();
            
    //         ctx.save();
    //         ctx.beginPath();
    //         ctx.rect(-50, -50, 100, 100 - (100 * (1 - percent)));
    //         ctx.clip();
    //         ctx.fillStyle = fillColor;
    //         drawHeartPath(ctx);
    //         ctx.fill();
    //         ctx.restore();
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         drawHeartPath(ctx);
    //         ctx.stroke();
    //         ctx.restore();
    //     },
    
    //     potion: (ctx, percent, emptyColor, fillColor) => {
    //         const size = 128;
    //         const center = 64;
    //         const bottleWidth = size * 0.5;
    //         const bottleHeight = size * 0.7;
    //         const bottleX = (size - bottleWidth) / 2;
    //         const bottleY = size * 0.15;
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         ctx.fillStyle = emptyColor;
            
    //         ctx.beginPath();
    //         ctx.moveTo(bottleX + bottleWidth * 0.2, bottleY);
    //         ctx.lineTo(bottleX + bottleWidth * 0.8, bottleY);
    //         ctx.lineTo(bottleX + bottleWidth, bottleY + bottleHeight * 0.2);
    //         ctx.lineTo(bottleX + bottleWidth, bottleY + bottleHeight);
    //         ctx.lineTo(bottleX, bottleY + bottleHeight);
    //         ctx.lineTo(bottleX, bottleY + bottleHeight * 0.2);
    //         ctx.closePath();
    //         ctx.fill();
    //         ctx.stroke();
            
    //         if (percent > 0) {
    //             const liquidHeight = bottleHeight * 0.8 * percent;
    //             const liquidY = bottleY + bottleHeight - liquidHeight;
    //             ctx.fillStyle = fillColor;
    //             ctx.fillRect(bottleX + 2, liquidY, bottleWidth - 4, liquidHeight - 2);
    //         }
            
    //         ctx.fillStyle = 'rgba(100,100,100,0.5)';
    //         ctx.fillRect(bottleX + bottleWidth * 0.35, bottleY - size * 0.1, bottleWidth * 0.3, size * 0.1);
    //         ctx.strokeRect(bottleX + bottleWidth * 0.35, bottleY - size * 0.1, bottleWidth * 0.3, size * 0.1);
    //     },
    
    //     orb: (ctx, percent, emptyColor, fillColor) => {
    //         const center = 64;
    //         const size = 128;
            
    //         let gradientColor = fillColor;
    //         let gradientColorSemi = fillColor;
    //         let gradientColorTransparent = fillColor;
            
    //         if (fillColor.startsWith('rgba')) {
    //             gradientColor = fillColor;
    //             const match = fillColor.match(/rgba\((\d+),\s*(\d+),\s*(\d+),\s*([\d.]+)\)/);
    //             if (match) {
    //                 const [, r, g, b, a] = match;
    //                 gradientColorSemi = `rgba(${r}, ${g}, ${b}, ${parseFloat(a) * 0.5})`;
    //                 gradientColorTransparent = `rgba(${r}, ${g}, ${b}, 0)`;
    //             }
    //         } else {
    //             gradientColor = fillColor;
    //             gradientColorSemi = fillColor + '88';
    //             gradientColorTransparent = fillColor + '00';
    //         }
            
    //         const gradient = ctx.createRadialGradient(center, center, 0, center, center, size * 0.5);
    //         gradient.addColorStop(0, gradientColor);
    //         gradient.addColorStop(0.7, gradientColorSemi);
    //         gradient.addColorStop(1, gradientColorTransparent);
            
    //         ctx.fillStyle = emptyColor;
    //         ctx.beginPath();
    //         ctx.arc(center, center, size * 0.4, 0, Math.PI * 2);
    //         ctx.fill();
            
    //         if (percent > 0) {
    //             ctx.fillStyle = gradient;
    //             ctx.globalAlpha = percent;
    //             ctx.beginPath();
    //             ctx.arc(center, center, size * 0.5, 0, Math.PI * 2);
    //             ctx.fill();
    //             ctx.globalAlpha = 1;
    //         }
            
    //         ctx.fillStyle = fillColor;
    //         ctx.beginPath();
    //         ctx.arc(center, center, size * 0.4 * percent, 0, Math.PI * 2);
    //         ctx.fill();
            
    //         ctx.fillStyle = 'rgba(255,255,255,0.4)';
    //         ctx.beginPath();
    //         ctx.arc(center - size * 0.1, center - size * 0.1, size * 0.15 * percent, 0, Math.PI * 2);
    //         ctx.fill();
    //     },
    
    //     diamond: (ctx, percent, emptyColor, fillColor) => {
    //         const center = 64;
    //         const size = 128;
    //         const dSize = size * 0.4;
            
    //         ctx.fillStyle = emptyColor;
    //         ctx.beginPath();
    //         ctx.moveTo(center, center - dSize);
    //         ctx.lineTo(center + dSize, center);
    //         ctx.lineTo(center, center + dSize);
    //         ctx.lineTo(center - dSize, center);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         if (percent > 0) {
    //             ctx.save();
    //             ctx.beginPath();
    //             const clipHeight = dSize * 2 * percent;
    //             const clipY = center + dSize - clipHeight;
    //             ctx.rect(center - dSize, clipY, dSize * 2, clipHeight);
    //             ctx.clip();
                
    //             ctx.fillStyle = fillColor;
    //             ctx.beginPath();
    //             ctx.moveTo(center, center - dSize);
    //             ctx.lineTo(center + dSize, center);
    //             ctx.lineTo(center, center + dSize);
    //             ctx.lineTo(center - dSize, center);
    //             ctx.closePath();
    //             ctx.fill();
    //             ctx.restore();
    //         }
            
    //         ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    //         ctx.lineWidth = 2;
    //         ctx.beginPath();
    //         ctx.moveTo(center, center - dSize);
    //         ctx.lineTo(center + dSize, center);
    //         ctx.lineTo(center, center + dSize);
    //         ctx.lineTo(center - dSize, center);
    //         ctx.closePath();
    //         ctx.stroke();
    //     },
    
    //     // Icon shapes (static, no percent fill)
    //     fire: (ctx) => {
    //         ctx.fillStyle = '#ff6b00';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 20);
    //         ctx.bezierCurveTo(80, 35, 85, 60, 85, 75);
    //         ctx.bezierCurveTo(85, 90, 76, 100, 64, 105);
    //         ctx.bezierCurveTo(52, 100, 43, 90, 43, 75);
    //         ctx.bezierCurveTo(43, 60, 48, 35, 64, 20);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#ffaa00';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 30);
    //         ctx.bezierCurveTo(74, 42, 77, 58, 77, 70);
    //         ctx.bezierCurveTo(77, 82, 71, 90, 64, 93);
    //         ctx.bezierCurveTo(57, 90, 51, 82, 51, 70);
    //         ctx.bezierCurveTo(51, 58, 54, 42, 64, 30);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#ffeb3b';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 40);
    //         ctx.bezierCurveTo(69, 48, 70, 58, 70, 66);
    //         ctx.bezierCurveTo(70, 74, 67, 80, 64, 82);
    //         ctx.bezierCurveTo(61, 80, 58, 74, 58, 66);
    //         ctx.bezierCurveTo(58, 58, 59, 48, 64, 40);
    //         ctx.closePath();
    //         ctx.fill();
    //     },
    
    //     ice: (ctx) => {
    //         ctx.fillStyle = '#4dd0e1';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 25);
    //         ctx.lineTo(85, 50);
    //         ctx.lineTo(75, 85);
    //         ctx.lineTo(64, 95);
    //         ctx.lineTo(53, 85);
    //         ctx.lineTo(43, 50);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#b3e5fc';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 25);
    //         ctx.lineTo(75, 50);
    //         ctx.lineTo(64, 60);
    //         ctx.lineTo(53, 50);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#80deea';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 60);
    //         ctx.lineTo(75, 50);
    //         ctx.lineTo(75, 85);
    //         ctx.lineTo(64, 95);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.beginPath();
    //         ctx.moveTo(64, 60);
    //         ctx.lineTo(53, 50);
    //         ctx.lineTo(53, 85);
    //         ctx.lineTo(64, 95);
    //         ctx.closePath();
    //         ctx.fill();
    //     },
    
    //     lightning: (ctx) => {
    //         ctx.fillStyle = '#ffeb3b';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 20);
    //         ctx.lineTo(50, 60);
    //         ctx.lineTo(70, 60);
    //         ctx.lineTo(55, 108);
    //         ctx.lineTo(78, 68);
    //         ctx.lineTo(60, 68);
    //         ctx.lineTo(75, 20);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#fff59d';
    //         ctx.beginPath();
    //         ctx.moveTo(67, 25);
    //         ctx.lineTo(58, 60);
    //         ctx.lineTo(68, 60);
    //         ctx.lineTo(62, 80);
    //         ctx.lineTo(72, 68);
    //         ctx.lineTo(65, 68);
    //         ctx.lineTo(71, 25);
    //         ctx.closePath();
    //         ctx.fill();
    //     },
    
    //     poison: (ctx) => {
    //         ctx.fillStyle = '#66bb6a';
    //         ctx.beginPath();
    //         ctx.arc(64, 55, 30, Math.PI, 0, false);
    //         ctx.lineTo(64, 95);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#388e3c';
    //         ctx.beginPath();
    //         ctx.arc(64, 55, 20, Math.PI, 0, false);
    //         ctx.lineTo(64, 85);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#a5d6a7';
    //         ctx.beginPath();
    //         ctx.arc(64, 55, 10, Math.PI, 0, false);
    //         ctx.lineTo(64, 70);
    //         ctx.closePath();
    //         ctx.fill();
    //     },
    
    //     magic: (ctx) => {
    //         ctx.fillStyle = '#f06292';
    //         ctx.beginPath();
            
    //         for (let i = 0; i < 5; i++) {
    //             const angle = (Math.PI * 2 / 5) * i - Math.PI / 2;
    //             const x = 64 + Math.cos(angle) * 45;
    //             const y = 64 + Math.sin(angle) * 45;
                
    //             if (i === 0) {
    //                 ctx.moveTo(x, y);
    //             } else {
    //                 ctx.lineTo(x, y);
    //             }
                
    //             const innerAngle = angle + (Math.PI / 5);
    //             const innerX = 64 + Math.cos(innerAngle) * 18;
    //             const innerY = 64 + Math.sin(innerAngle) * 18;
    //             ctx.lineTo(innerX, innerY);
    //         }
            
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#f8bbd0';
    //         ctx.beginPath();
            
    //         for (let i = 0; i < 5; i++) {
    //             const angle = (Math.PI * 2 / 5) * i - Math.PI / 2;
    //             const x = 64 + Math.cos(angle) * 25;
    //             const y = 64 + Math.sin(angle) * 25;
                
    //             if (i === 0) {
    //                 ctx.moveTo(x, y);
    //             } else {
    //                 ctx.lineTo(x, y);
    //             }
                
    //             const innerAngle = angle + (Math.PI / 5);
    //             const innerX = 64 + Math.cos(innerAngle) * 10;
    //             const innerY = 64 + Math.sin(innerAngle) * 10;
    //             ctx.lineTo(innerX, innerY);
    //         }
            
    //         ctx.closePath();
    //         ctx.fill();
    //     },
    
    //     physical: (ctx) => {
    //         ctx.fillStyle = '#9e9e9e';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 20);
    //         ctx.lineTo(70, 75);
    //         ctx.lineTo(64, 80);
    //         ctx.lineTo(58, 75);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#e0e0e0';
    //         ctx.beginPath();
    //         ctx.moveTo(64, 20);
    //         ctx.lineTo(66, 75);
    //         ctx.lineTo(64, 78);
    //         ctx.lineTo(62, 75);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillStyle = '#6d4c41';
    //         ctx.fillRect(45, 75, 38, 6);
            
    //         ctx.fillStyle = '#8d6e63';
    //         ctx.fillRect(58, 81, 12, 18);
            
    //         ctx.strokeStyle = '#6d4c41';
    //         ctx.lineWidth = 1.5;
    //         for (let i = 0; i < 4; i++) {
    //             ctx.beginPath();
    //             ctx.moveTo(58, 84 + i * 4);
    //             ctx.lineTo(70, 84 + i * 4);
    //             ctx.stroke();
    //         }
            
    //         ctx.fillStyle = '#6d4c41';
    //         ctx.beginPath();
    //         ctx.arc(64, 102, 5, 0, Math.PI * 2);
    //         ctx.fill();
    //     },
    
    //     sfx: (ctx) => {
    //         ctx.fillStyle = '#ffa726';
    //         ctx.beginPath();
    //         ctx.moveTo(45, 50);
    //         ctx.lineTo(45, 78);
    //         ctx.lineTo(60, 70);
    //         ctx.lineTo(60, 58);
    //         ctx.closePath();
    //         ctx.fill();
            
    //         ctx.fillRect(35, 54, 10, 20);
            
    //         ctx.strokeStyle = '#ffb74d';
    //         ctx.lineWidth = 4;
    //         ctx.lineCap = 'round';
            
    //         ctx.beginPath();
    //         ctx.arc(60, 64, 12, -Math.PI/4, Math.PI/4, false);
    //         ctx.stroke();
            
    //         ctx.beginPath();
    //         ctx.arc(60, 64, 22, -Math.PI/4, Math.PI/4, false);
    //         ctx.stroke();
            
    //         ctx.beginPath();
    //         ctx.arc(60, 64, 32, -Math.PI/4, Math.PI/4, false);
    //         ctx.stroke();
    //     },
    
    //     fx: (ctx) => {
    //         ctx.fillStyle = '#ab47bc';
    //         ctx.beginPath();
    //         ctx.arc(64, 64, 12, 0, Math.PI * 2);
    //         ctx.fill();
            
    //         const rays = 8;
    //         for (let i = 0; i < rays; i++) {
    //             const angle = (Math.PI * 2 / rays) * i;
    //             const x1 = 64 + Math.cos(angle) * 15;
    //             const y1 = 64 + Math.sin(angle) * 15;
    //             const x2 = 64 + Math.cos(angle) * 40;
    //             const y2 = 64 + Math.sin(angle) * 40;
                
    //             const angleLeft = angle - 0.3;
    //             const angleRight = angle + 0.3;
    //             const x3 = 64 + Math.cos(angleLeft) * 35;
    //             const y3 = 64 + Math.sin(angleLeft) * 35;
    //             const x4 = 64 + Math.cos(angleRight) * 35;
    //             const y4 = 64 + Math.sin(angleRight) * 35;
                
    //             ctx.beginPath();
    //             ctx.moveTo(x1, y1);
    //             ctx.lineTo(x3, y3);
    //             ctx.lineTo(x2, y2);
    //             ctx.lineTo(x4, y4);
    //             ctx.closePath();
    //             ctx.fill();
    //         }
            
    //         ctx.fillStyle = '#ce93d8';
    //         ctx.beginPath();
    //         ctx.arc(64, 64, 8, 0, Math.PI * 2);
    //         ctx.fill();
    //     }
        
    // };
    
    // shapes.metadata = {
    //     fire: { name: 'Fire', color: '#ff4500' },
    //     ice: { name: 'Ice', color: '#00bfff' },
    //     lightning: { name: 'Lightning', color: '#ffd700' },
    //     poison: { name: 'Poison', color: '#4caf50' },
    //     magic: { name: 'Magic', color: '#e91e63' },
    //     physical: { name: 'Physical', color: '#795548' },
    //     sfx: { name: 'SFX', color: '#ff9800' },
    //     fx: { name: 'FX', color: '#9c27b0' }
    // };
    // ==============================================
    // SPRITE CONFIGURATION
    // ==============================================
    const sprite = {
        generateConfig(config) {
            const json = {
                frame_count: config.frameCount,
                fps: config.fps || 15.0,
                layout: config.layout || 'horizontal'
            };

            if (config.layout === 'grid' && config.gridColumns) {
                json.grid_columns = config.gridColumns;
            }

            if (config.speed && config.speed !== 1.0) {
                json.speed = config.speed;
            }

            if (config.animationMode) {
                json.animation_mode = config.animationMode;
            }

            return json;
        },

        validateConfig(json) {
            const errors = [];

            if (!json.frame_count || typeof json.frame_count !== 'number') {
                errors.push('Missing or invalid frame_count');
            }

            if (json.layout && !['horizontal', 'vertical', 'grid'].includes(json.layout)) {
                errors.push('Invalid layout type');
            }

            if (json.layout === 'grid' && !json.grid_columns) {
                errors.push('grid_columns required for grid layout');
            }

            if (json.fps && (typeof json.fps !== 'number' || json.fps <= 0)) {
                errors.push('Invalid fps value');
            }

            return {
                valid: errors.length === 0,
                errors
            };
        },

        generateRustCode(config) {
            const { layout, frameCount, frameWidth, frameHeight, gridColumns } = config;
            
            let layoutType;
            if (layout === 'horizontal') {
                layoutType = 'HorizontalStrip';
            } else if (layout === 'vertical') {
                layoutType = 'VerticalStrip';
            } else {
                layoutType = `Grid { columns: ${gridColumns} }`;
            }

            return `// For your sprite sheet:
SpriteSheetLayout::${layoutType}

// Frame dimensions: ${frameWidth}x${frameHeight}
// Total frames: ${frameCount}

// In ImageLoader::load_frames():
let layout = Some(SpriteSheetLayout::${layoutType});
ImageLoader::load_frames(
    base_paths,
    "your_folder_name",
    ${frameCount}, // frame_count
    game_loader,
    layout
);`;
        }
    };

    // ==============================================
    // STORAGE UTILITIES
    // ==============================================
    const storage = {
        get(key, defaultValue = null) {
            try {
                const item = localStorage.getItem(key);
                return item ? JSON.parse(item) : defaultValue;
            } catch (e) {
                console.warn('LocalStorage not available:', e);
                return defaultValue;
            }
        },

        set(key, value) {
            try {
                localStorage.setItem(key, JSON.stringify(value));
                return true;
            } catch (e) {
                console.warn('LocalStorage not available:', e);
                return false;
            }
        },

        remove(key) {
            try {
                localStorage.removeItem(key);
                return true;
            } catch (e) {
                console.warn('LocalStorage not available:', e);
                return false;
            }
        },

        clear() {
            try {
                localStorage.clear();
                return true;
            } catch (e) {
                console.warn('LocalStorage not available:', e);
                return false;
            }
        }
    };

    // ==============================================
    // MATH UTILITIES
    // ==============================================
    const math = {
        clamp(value, min, max) {
            return Math.min(Math.max(value, min), max);
        },

        lerp(a, b, t) {
            return a + (b - a) * t;
        },

        map(value, inMin, inMax, outMin, outMax) {
            return (value - inMin) * (outMax - outMin) / (inMax - inMin) + outMin;
        },

        degToRad(degrees) {
            return degrees * (Math.PI / 180);
        },

        radToDeg(radians) {
            return radians * (180 / Math.PI);
        }
    };

    // ==============================================
    // UI UTILITIES
    // ==============================================
    const ui = {
        showNotification(message, type = 'info', duration = 3000) {
            const notification = document.createElement('div');
            notification.className = `notification notification-${type}`;
            notification.textContent = message;
            notification.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                padding: 1rem 1.5rem;
                background: ${type === 'error' ? '#ef4444' : type === 'success' ? '#10b981' : type === 'warning' ? '#f59e0b' : '#3b82f6'};
                color: white;
                border-radius: 8px;
                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                z-index: 10000;
                animation: slideIn 0.3s ease-out;
            `;
            
            document.body.appendChild(notification);
            
            setTimeout(() => {
                notification.style.animation = 'slideOut 0.3s ease-out';
                setTimeout(() => notification.remove(), 300);
            }, duration);
        },

        createButton(text, onClick, className = '') {
            const button = document.createElement('button');
            button.textContent = text;
            button.className = className;
            button.onclick = onClick;
            return button;
        },

        createElement(tag, props = {}, children = []) {
            const el = document.createElement(tag);
            
            Object.entries(props).forEach(([key, value]) => {
                if (key === 'className') {
                    el.className = value;
                } else if (key === 'style' && typeof value === 'object') {
                    Object.assign(el.style, value);
                } else if (key.startsWith('on') && typeof value === 'function') {
                    el.addEventListener(key.substring(2).toLowerCase(), value);
                } else {
                    el[key] = value;
                }
            });
            
            children.forEach(child => {
                if (typeof child === 'string') {
                    el.appendChild(document.createTextNode(child));
                } else if (child instanceof Node) {
                    el.appendChild(child);
                }
            });
            
            return el;
        }
    };

    // ==============================================
    // UTILITY FUNCTIONS
    // ==============================================
    const utils = {
        debounce(func, wait) {
            let timeout;
            return function executedFunction(...args) {
                const later = () => {
                    clearTimeout(timeout);
                    func(...args);
                };
                clearTimeout(timeout);
                timeout = setTimeout(later, wait);
            };
        },

        throttle(func, limit) {
            let inThrottle;
            return function(...args) {
                if (!inThrottle) {
                    func.apply(this, args);
                    inThrottle = true;
                    setTimeout(() => inThrottle = false, limit);
                }
            };
        },

        uuid() {
            return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                const r = Math.random() * 16 | 0;
                const v = c === 'x' ? r : (r & 0x3 | 0x8);
                return v.toString(16);
            });
        }
    };

    // ==============================================
    // EXPORT MODULE
    // ==============================================
    const SpriteTools = {
        color,
        canvas,
        files,
        frames,
        // shapes,
        sprite,
        storage,
        math,
        ui,
        utils,
        version: '1.0.0'
    };

    // Support multiple export formats
    if (typeof module !== 'undefined' && module.exports) {
        module.exports = SpriteTools;
    } else if (typeof define === 'function' && define.amd) {
        define([], function() { return SpriteTools; });
    } else {
        global.SpriteTools = SpriteTools;
    }

})(typeof window !== 'undefined' ? window : this);
/**
 * Converts a long string of bytes into a readable format e.g KB, MB, GB, TB, YB
 * 
 * @param {number} bytes The number of bytes.
 */
function readableBytes(bytes: number) {
    console.log('readableBytes called with:', bytes);
    if (typeof bytes !== 'number' || isNaN(bytes) || bytes < 0) return '';
    if (bytes === 0) return '0 B';
    var i = Math.floor(Math.log(bytes) / Math.log(1024)),
    sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${sizes[i]}`;
}

function compareVersions(a: any, b: any): number {
    const aParts = a.version.split('.').map(Number);
    const bParts = b.version.split('.').map(Number);
    for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
      const aPart = aParts[i] || 0;
      const bPart = bParts[i] || 0;
      if (aPart > bPart) return -1;
      if (aPart < bPart) return 1;
    }
    return 0;
}

export {
    readableBytes,
    compareVersions
}
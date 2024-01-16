class FixedArray4 {
    constructor(array) {
        this.array = array;
    }

    // 从字符串转换为 FixedArray4 的静态方法
    static fromString(s) {
        const cleaned = s.startsWith("0x") ? s.slice(2) : s;
        let result = new Array(4).fill(0);
        // 实现转换逻辑
        // 示例逻辑，可能需要根据实际情况调整
        let chunks = cleaned.match(/.{1,16}/g) || [];
        chunks = chunks.reverse();
        for (let i = 0; i < chunks.length; i++) {
            result[i] = parseInt(chunks[i], 16);
        }
        return new FixedArray4(result);
    }

    // 转换为十六进制字符串的方法
    toHexString() {
        let hexString = "0x";
        for (const value of this.array) {
            hexString += value.toString(16).padStart(16, '0');
        }
        return hexString;
    }
}

// 示例使用
const fixedArray = FixedArray4.fromString("0x1234567890abcdef");
console.log(fixedArray.toHexString()); // 输出转换后的十六进制字符串

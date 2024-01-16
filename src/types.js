const Type = {
    U32: 'U32',
    Field: 'Field',
    Hash: 'Hash',
    Address: 'Address',
    Bool: 'Bool',
    FixedArray: 'FixedArray',
    String: 'String',
    Fields: 'Fields',
    Array: 'Array',
    Tuple: 'Tuple',
    // 其他类型
};

function isDynamic(type) {
    switch (type) {
        case Type.U32:
        case Type.Field:
        case Type.Address:
        case Type.Hash:
        case Type.Bool:
            return false;
        case Type.FixedArray:
            // 处理 FixedArray 的逻辑
            return isDynamic(/* 嵌套类型 */);
        case Type.String:
        case Type.Fields:
        case Type.Array:
            return true;
        // 其他类型的处理
        default:
            return false;
    }
}


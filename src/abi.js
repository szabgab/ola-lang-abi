class Abi {
    constructor(functions) {
        this.functions = functions;
    }
    
    findFunctionByInput(input) {
        const functionMatch = this.functions.find(func => func.methodId === input[input.length - 1]);
        if (!functionMatch) {
            throw new Error("ABI function not found");
        }
        return functionMatch;
    }
}


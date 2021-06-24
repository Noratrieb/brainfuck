type InHandler = (() => number);
type OutHandler = ((char: number) => void);
type ErrorHandler = ((msg: string) => void);

export default class Interpreter {
    private readonly _array: Uint8Array;
    private _pointer: number;
    private readonly _code: string;
    private _codePointer: number;

    private readonly _inHandler: InHandler;
    private readonly _outHandler: OutHandler;
    private readonly _errorHandler: ErrorHandler;

    constructor(code: string, outHandler: OutHandler, inHandler: InHandler, errorHandler: ErrorHandler) {
        const buf = new ArrayBuffer(32000);
        this._array = new Uint8Array(buf);
        this._pointer = 0;
        this._code = code;
        this._codePointer = 0;
        this._inHandler = inHandler;
        this._outHandler = outHandler;
        this._errorHandler = errorHandler;
    }

    public next() {
        switch (this._code[this._codePointer++]) {
            case '+':
                this._array[this._pointer]++;
                break;
            case '-':
                this._array[this._pointer]--;
                break;
            case '>':
                this._pointer++;
                break;
            case '<':
                if (this._pointer === 0) {
                    this._errorHandler("Cannot wrap left");
                    break;
                }
                this._pointer--;
                break;
            case '.':
                this._outHandler(this.value);
                break;
            case ',':
                this._array[this._pointer] = this._inHandler();
                break;
            case '[':
                if (this.value === 0) {
                    let level = 0;
                    while (this.lastInstruction !== ']' || level > -1) {
                        this._codePointer++;
                        if (this.lastInstruction === '[') level++;
                        else if (this.lastInstruction === ']') level--;
                    }
                }
                break;
            case ']':
                if (this.value !== 0) {
                    let level = 0;
                    while (this.lastInstruction !== '[' || level > -1) {
                        this._codePointer--;
                        if (this.lastInstruction === '[') level--;
                        else if (this.lastInstruction === ']') level++;
                    }
                }
                break;
            case undefined:
                this._pointer--;
                console.warn("reached end");
                break;
            default: {
            }
        }
        console.log(`char: ${this.code[this.codePointer - 1]}  pointer: ${this.pointer} value: ${this.array[this.pointer]}`)
    }

    public prev() {

    }

    get reachedEnd(): boolean {
        return this._codePointer === this._code.length - 1;
    }

    get lastInstruction(): string {
        return this._code[this._codePointer - 1];
    }

    get value(): number {
        return this._array[this._pointer];
    }

    get array(): Uint8Array {
        return this._array;
    }

    get pointer(): number {
        return this._pointer;
    }

    get code(): string {
        return this._code;
    }

    get codePointer(): number {
        return this._codePointer;
    }
}
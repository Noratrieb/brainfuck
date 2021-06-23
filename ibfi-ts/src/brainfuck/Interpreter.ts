export default class Interpreter {
    private readonly _array: Uint8Array;
    private _pointer: number;
    private readonly _code: string;
    private _codePointer: number;

    private _inHandler: (() => number);
    private _outHandler: ((char: number) => void);

    constructor(code: string, outHandler: ((char: number) => void), inHandler: (() => number)) {
        const buf = new ArrayBuffer(32000);
        this._array = new Uint8Array(buf);
        this._pointer = 0;
        this._code = code;
        this._codePointer = 0;
        this._inHandler = inHandler;
        this._outHandler = outHandler;
    }

    public next() {
        switch (this._code[++this._codePointer]) {
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
                this._pointer--;
                break;
            case '.':
                this._outHandler(this._array[this._pointer]);
                break;
            case ',':
                this._array[this._pointer] = this._inHandler();
                break;
            case '[':
                console.error("does not support [ for now")
                break;
            case ']':
                console.error("does not support ] for now")
                break;
            default: {
            }
        }
        console.log("next step")
    }

    public prev() {

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
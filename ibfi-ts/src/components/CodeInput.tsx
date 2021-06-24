import React, {useState} from 'react';

export interface CodeOptions {
    minify?: boolean
}

interface CodeInputProps {
    setInput: ((code: string, options: CodeOptions) => void),
    code: string
}

const CodeInput = ({code, setInput}: CodeInputProps) => {
    const [fontSize, setFontSize] = useState(40);

    const [codeOptions, setCodeOptions] = useState<CodeOptions>({});


    const setStart = () => {
        setInput(
            "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.",
            codeOptions);
    }

    const changeMinify = (e: React.ChangeEvent<HTMLInputElement>) => {
        setCodeOptions(old => ({...old, minify: e.target.checked}))
        setInput(code, codeOptions);
    }

    return (
        <div>
            <div className="bf-input">
                <div>
                <span>
                    <label htmlFor="bf-input-fontsize-range">Font Size</label>
                    <input type="range" id="bf-input-fontsize-range" onChange={v => setFontSize(+v.target.value)}/>
                </span>
                    <input type="checkbox" checked={codeOptions.minify} id="input-options-minify" onChange={changeMinify}/>
                    <label htmlFor="input-options-minify">Minify Code</label>
                </div>
                <textarea value={code} onChange={e => setInput(e.target.value, codeOptions)} style={{fontSize}}
                          className="code-input"
                          placeholder="Input your code here..."/>
                <div>
                    <button onClick={setStart}>Set Hello World</button>
                </div>
            </div>
        </div>
    );
};

export default CodeInput;
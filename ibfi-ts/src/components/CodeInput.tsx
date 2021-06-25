import React, {useState} from 'react';
import presets from "../presets.json";

export interface CodeOptions {
    minify?: boolean,
    directStart?: boolean,
    enableBreakpoints?: boolean
}

interface CodeInputProps {
    setInput: ((code: string, options: CodeOptions) => void),
    code: string
}

const CodeInput = ({code, setInput}: CodeInputProps) => {
    const [fontSize, setFontSize] = useState(40);

    const [codeOptions, setCodeOptions] = useState<CodeOptions>({});


    const setPreset = (name: keyof typeof presets) => () => {
        setInput(presets[name], codeOptions);
    }

    const changeMinify = (e: React.ChangeEvent<HTMLInputElement>) => {
        setCodeOptions(old => ({...old, minify: e.target.checked}))
        setInput(code, codeOptions);
    }

    const changeStart = (e: React.ChangeEvent<HTMLInputElement>) => {
        setCodeOptions(old => ({...old, directStart: e.target.checked}))
        setInput(code, codeOptions);
    }

    const changeBreakpoint = (e: React.ChangeEvent<HTMLInputElement>) => {
        setCodeOptions(old => ({...old, enableBreakpoints: e.target.checked}))
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
                    <span>
                    <input type="checkbox" checked={codeOptions.minify} id="input-options-minify"
                           onChange={changeMinify}/>
                    <label htmlFor="input-options-minify">Minify Code</label>
                    </span>
                    <span>
                    <input type="checkbox" checked={codeOptions.directStart} id="input-options-directstart"
                           onChange={changeStart}/>
                    <label htmlFor="input-options-directstart">Start Directly</label>
                    </span>
                    <span>
                    <input type="checkbox" checked={codeOptions.enableBreakpoints} id="input-options-enableBreakpoints"
                           onChange={changeBreakpoint}/>
                    <label htmlFor="input-options-enableBreakpoints">Breakpoints (•)</label>
                    </span>
                </div>
                <textarea value={code} onChange={e => setInput(e.target.value, codeOptions)} style={{fontSize}}
                          className="code-input"
                          placeholder="Input your code here..."/>
                <div>
                    <div>Presets</div>
                    <div>
                        <button onClick={setPreset("helloworld")}>Hello World</button>
                        <button onClick={setPreset("hanoi")}>Towers of Hanoi</button>
                        <button onClick={setPreset("quine")}>Quine</button>
                        <button onClick={setPreset("gameoflife")}>Game Of Life</button>
                        <button onClick={setPreset("benchmark")}>Benchmark</button>
                        <button onClick={setPreset("fizzbuzz")}>Fizzbuzz</button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default CodeInput;
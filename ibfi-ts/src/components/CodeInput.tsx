import React, {ChangeEvent, useState} from 'react';
import presets from "../presets.json";

export interface CodeOptions {
    minify?: boolean,
    directStart?: boolean,
    enableBreakpoints?: boolean
    asciiView?: boolean
}

interface CodeInputProps {
    setInput: ((code: string, options: CodeOptions) => void),
    input: [string, CodeOptions]
}

const CodeInput = ({input: [code, options], setInput}: CodeInputProps) => {
    const [fontSize, setFontSize] = useState(40);

    const setPreset = (name: keyof typeof presets) => () => {
        setInput(presets[name], options);
    }

    const changeHandler = (name: keyof CodeOptions) => (event: ChangeEvent<HTMLInputElement>) => {
        setInput(code, {...options, [name]: event.target.checked})
    }

    return (
        <div className="bf-input">
            <div>
                <div>
                    <label htmlFor="bf-input-fontsize-range">Font Size</label>
                    <input type="range" id="bf-input-fontsize-range" onChange={v => setFontSize(+v.target.value)}/>
                </div>

                <CodeOption displayName="Minify Code" name="minify" options={options} onChange={changeHandler}/>
                <CodeOption displayName="Start Directly" name="directStart" options={options}
                            onChange={changeHandler}/>
                <CodeOption displayName="Breakpoints (•)" name="enableBreakpoints" options={options}
                            onChange={changeHandler}/>
                <CodeOption displayName="Show ASCII in memory" name="asciiView" options={options}
                            onChange={changeHandler}/>

            </div>
            <textarea value={code} onChange={e => setInput(e.target.value, options)} style={{fontSize}}
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
    );
};

interface CodeOptionProps {
    displayName: string,
    name: keyof CodeOptions,
    options: CodeOptions,
    onChange: (name: keyof CodeOptions) => (event: ChangeEvent<HTMLInputElement>) => void,
}

const CodeOption = ({displayName, name, options, onChange}: CodeOptionProps) => {
    return (
        <span>
        <input type="checkbox" checked={options[name]} id={`input-options-${name}`}
               onChange={onChange(name)}/>
        <label htmlFor={`input-options-${name}`}>{displayName}</label>
        </span>
    );
}

export default CodeInput;
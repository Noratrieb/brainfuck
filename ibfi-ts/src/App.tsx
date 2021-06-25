import './App.scss';
import CodeInput, {CodeOptions} from "./components/CodeInput";
import ProgramOutput from "./components/ProgramOutput";
import React, {useCallback, useState} from "react";
import Runner from "./components/Runner";

export const OptionContext = React.createContext<CodeOptions>({});

function App() {
    const [out, setOut] = useState("");
    const [input, setInput] = useState<[string, CodeOptions]>(["", {}]);
    const [running, setRunning] = useState(false);

    const outHandler = useCallback((char: number) => {
        setOut(oldOut => oldOut + String.fromCharCode(char))
    }, []);

    const runHandler = (run: boolean) => {
        setRunning(run);
        if (!run) {
            setOut("");
        }
    }


    const inputHandler = (code: string, options: CodeOptions) => setInput([code, options]);
    return (
        <div className="App-header">
            <OptionContext.Provider value={input[1]}>
                {
                    !running && <CodeInput input={input} setInput={inputHandler}/>
                }
                <Runner running={running} setRunning={runHandler} code={input[0]} outHandler={outHandler}/>
                {
                    running && <ProgramOutput text={out}/>
                }
            </OptionContext.Provider>
        </div>
    );
}

export default App;
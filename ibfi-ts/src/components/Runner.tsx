import React, {useCallback, useEffect, useRef, useState} from 'react';
import Interpreter from "../brainfuck/Interpreter";
import CodeDisplay from "./CodeDisplay";
import RunDisplay from "./RunDisplay";
import {CodeOptions} from "./CodeInput";

interface RunInfoProps {
    input: [string, CodeOptions],
    setRunning: (running: boolean) => void,
    running: boolean
    outHandler: (char: number) => void,
}

const Runner = ({setRunning, running, outHandler, input}: RunInfoProps) => {
    const [speed, setSpeed] = useState(0);
    const [interpreter, setInterpreter] = useState<Interpreter | null>(null);
    const [error, setError] = useState<string | null>(null);

    const [, setRerenderNumber] = useState(0);

    const inputArea = useRef<HTMLTextAreaElement>(null);


    const inputHandler = () => {
        if (!inputArea.current) {
            throw new Error("Could not read input")
        }
        const value = inputArea.current.value;
        if (value.length < 1) {
            throw new Error("No input found");
        }
        const char = value.charCodeAt(0);
        inputArea.current.value = value.substr(1);
        return char;
    }

    const errorHandler = (msg: string) => setError(msg);

    const startHandler = useCallback(() => {
        setSpeed(0);
        setInterpreter(new Interpreter(input, outHandler, inputHandler, errorHandler));
        setRunning(false);
        setRunning(true);
    }, [input, outHandler, setRunning]);

    const stopHandler = () => setRunning(false);

    const nextHandler = useCallback(() => {
        setError(null);
        interpreter?.next();
        if (interpreter?.reachedEnd) {
            setSpeed(0);
        }
        setRerenderNumber(n => n + 1);
    }, [interpreter]);

    useEffect(() => {
        if (running) {
            if (speed === 0) {
                return;
            }
            const interval = setInterval(() => {
                nextHandler();
            }, 1000 / (speed));

            return () => clearInterval(interval);
        }
    }, [running, nextHandler, speed]);


    return (
        <div className="bf-run">
            {
                running && interpreter && <>
                    <CodeDisplay code={interpreter.code} index={interpreter.programCounter}/>
                    <RunDisplay interpreter={interpreter}/>
                </>
            }
            <div>
                <button onClick={stopHandler}>Back</button>
                <button onClick={startHandler}>Start</button>
                <button onClick={nextHandler}>Next</button>
            </div>
            {
                running && <>
                    <div>
                        <label htmlFor="run-info-speed-range">Speed</label>
                        <input type="range" id="run-info-speed-range" value={speed}
                               onChange={e => setSpeed(+e.target.value)}/>
                        <span> {speed}</span>
                    </div>
                </>
            }
            {
                error && <div className="error">{error}</div>
            }
            {
                running && <div>
                    <div>Input:</div>
                    <textarea className="program-input-area" ref={inputArea}/>
                </div>
            }
        </div>
    );
};

export default Runner;
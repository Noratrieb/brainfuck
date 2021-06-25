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
    const [info, setInfo] = useState<string | null>(null);
    const [startTime, setStartTime] = useState(0);


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

    const startHandler = useCallback(() => {
        if (input[1].directStart) {
            setSpeed(100);
        } else {
            setSpeed(0);
        }

        setStartTime(Date.now);
        setInterpreter(new Interpreter(input, outHandler, inputHandler));
        setRunning(false);
        setRunning(true);
    }, [input, outHandler, setRunning]);

    const stopHandler = () => {
        setRunning(false);
        setInfo(null);
    }

    const nextHandler = useCallback(() => {
        setInfo(null);
        try {
            interpreter?.next();
        } catch (e) {
            setInfo(e.message);
            setSpeed(0);
        }
        if (interpreter?.reachedEnd) {
            setSpeed(0);
            setInfo(`Finished Execution. Took ${(Date.now() - startTime) / 1000}s`)
        }
        setRerenderNumber(n => n + 1);
    }, [interpreter, startTime]);

    useEffect(() => {
        if (running) {
            if (speed === 0) {
                return;
            }

            const interval = setInterval(() => {
                nextHandler();
            }, 1000 / (speed * 10));

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
                {running && <button onClick={stopHandler}>Back</button>}
                <button onClick={startHandler}>{running ? "Restart" : "Start"}</button>
                {running && <button onClick={nextHandler}>Next</button>}
            </div>
            {
                running &&
                <div>
                    <label htmlFor="run-info-speed-range">Speed</label>
                    <input type="range" id="run-info-speed-range" value={speed}
                           onChange={e => setSpeed(+e.target.value)}/>
                    <span> {speed}</span>
                    <span>
                            <button onClick={() => setSpeed(s => s === 0 ? 0 : s - 1)}
                                    className="small-speed-button">-</button>
                            <button onClick={() => setSpeed(0)}
                                    className="small-speed-button">0</button>
                            <button onClick={() => setSpeed(s => s === 100 ? 100 : s + 1)}
                                    className="small-speed-button">+</button>
                        </span>
                </div>
            }
            {info && <div className="info">{info}</div>}
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
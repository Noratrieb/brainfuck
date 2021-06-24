import React, {useCallback, useEffect, useState} from 'react';
import Interpreter from "../brainfuck/Interpreter";
import CodeDisplay from "./CodeDisplay";
import RunDisplay from "./RunDisplay";

interface RunInfoProps {
    input: string,
    setRunning: (running: boolean) => void,
    running: boolean
    inHandler: () => number,
    outHandler: (char: number) => void,
}

const Runner = ({setRunning, running, inHandler, outHandler, input}: RunInfoProps) => {
        const [speed, setSpeed] = useState(0);
        const [interpreter, setInterpreter] = useState<Interpreter | null>(null);

        const [, setRerenderNumber] = useState(0);

        const startHandler = useCallback(() => {
            setSpeed(0);
            setInterpreter(new Interpreter(input, outHandler, inHandler));
            setRunning(true);
        }, [input, inHandler, outHandler, setRunning]);

        const stopHandler = () => setRunning(false);

        const nextHandler = useCallback(() => {
            interpreter?.next();
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
                {running && interpreter && <>
                    <CodeDisplay code={input} index={interpreter.codePointer}/>
                    <RunDisplay interpreter={interpreter}/>
                </>
                }
                <div>
                    <button onClick={startHandler}>Start</button>
                    <button onClick={stopHandler}>Stop</button>
                    <button onClick={nextHandler}>Next</button>
                </div>
                {
                    running &&
                    <>
                        <div>
                            <label htmlFor="run-info-speed-range">Speed</label>
                            <input type="range" id="run-info-speed-range" value={speed}
                                   onChange={e => setSpeed(+e.target.value)}/>
                            <span> {speed}</span>
                        </div>
                    </>
                }
            </div>
        );
    }
;

export default Runner;
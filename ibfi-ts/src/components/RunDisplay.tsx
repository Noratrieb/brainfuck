import React from 'react';
import Interpreter from "../brainfuck/Interpreter";

const MAX_TABLE_COLUMNS = 30;

interface RunDisplayProps {
    interpreter: Interpreter,
}

const RunDisplay = ({interpreter}: RunDisplayProps) => {

    const index = interpreter.pointer;

    let offset: number;

    if (index < MAX_TABLE_COLUMNS / 2) {
        offset = 0;
    } else {
        offset = index - MAX_TABLE_COLUMNS / 2;
    }

    const arrayWithIndex = Array(MAX_TABLE_COLUMNS).fill(0)
        .map((_, i) => i + offset);

    return (
        <div>
            <table className="memory-display-table">
                <thead>
                <tr>
                    {
                        arrayWithIndex.map((n => <th key={n}>{n}</th>))
                    }
                </tr>
                </thead>
                <tbody>
                <tr>
                    {
                        arrayWithIndex.map((n) => <td className="cell" key={n}>{interpreter.array[n]}</td>)
                    }
                </tr>
                <tr>
                    {
                        arrayWithIndex.map((n) => <td className="pointer"
                                                      key={n}>{interpreter.pointer === n && "^"}</td>)
                    }
                </tr>
                </tbody>
            </table>
        </div>
    );
};

export default RunDisplay;

export class AssertionError extends Error {
    cause;
    notes;

    constructor(cause, notes) {
        let name = "Assertion Error";
        let note = notes.map((p) => "\n    note: " + p).join("");
        let message = name + ": " + cause + note;
        super(message);
        this.cause = cause;
        this.notes = notes;
    }
}

export class Assert {
    /**
     * 
     * @param {T} left 
     * @param {T} right 
     */
    static assertEqual(left, right) {
        if (left !== right) {
            throw new AssertionError("left should equal right", [
                "left is '" + left.toString() + "'",
                "right is '" + right.toString() + "'",
            ]);
        }
    }

    /**
     * 
     * @param {T} left 
     * @param {T} right 
     */
    static assertNotEqual(left, right) {
        if (left === right) {
            throw new AssertionError("left should not equal right", [
                "left is '" + left.toString() + "'",
                "right is '" + right.toString() + "'",
            ]);
        }
    }

    /**
     * 
     * @param {boolean} cond 
     */
    static assert(cond) {
        if (!cond) {
            throw new AssertionError("cond should be true", []);
        }
    }

    /**
     * 
     * @param {any} obj 
     */
    static assertNotNull(obj) {
        if (obj === null) {
            throw new AssertionError("obj should not be null", ["obj was null"]);
        } else if (obj === undefined) {
            throw new AssertionError("obj should not be null", ["obj was undefined"]);
        } else if (typeof obj === "undefined") {
            throw new AssertionError("obj should not be null", ["typeof obj was \"undefined\""]);
        }
    }
}
import axios from "axios";
import { Match } from "./types";
import assert, { deepEqual } from "assert";

axios.defaults.validateStatus = (status) => {
    return true;
}
const letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabdefghijklmnopqrstuvwxyz"
function generateRandomToken(): string {
    return Array.from({ length: 20 }).map((_) => letters[Math.floor(Math.random() * letters.length)]).join("");
}
const PRIZES: number[] = [2, 5, 10, 25, 50]
const GAME_TYPES: string[] = ["soccer", "knockout"]
function pickRandom<T>(arr: T[]): T {
    return arr[Math.floor(Math.random() * arr.length)];
}
const URL = "http://localhost:8080"
class Client {
    token: string
    constructor() {
        this.token = generateRandomToken();
    }
    async createGame(): Promise<Match> {
        const prize = pickRandom(PRIZES)
        const game_type = pickRandom(GAME_TYPES)
        const response = await axios.post(`${URL}/create`,
            { prize, game_type },
            {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${this.token}`,
                },
            }
        )
        assert(response.status === 200, "Invalid response status");
        const { id, players, prize: gamePrize, game_type: gameGame_type, expiry_time } = response.data as Match;
        assert(players.includes(this.token), "Players does not include token");
        assert(Number(gamePrize) === prize, "Incorrect prize value");
        assert(gameGame_type === game_type, "Invalid game type");
        return response.data as Match;
    }
    async joinGame(id: number): Promise<Match> {
        const response = await axios.post(`${URL}/join?id=${id}`, null,
            {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${this.token}`
                },
            }
        )
        assert(response.status === 200, "Invalid response status");
        const { id: game_id, game_type, prize, expiry_time, players } = response.data as Match;
        assert(game_id === id, "Invalid game id");
        return response.data as Match
    }
    async getGame(id: number): Promise<Match> {
        const response = await axios.get(`${URL}/match?id=${id}`,
            {
                method: "GET",
                headers: {
                    Authorization: `Bearer ${this.token}`,
                }
            }
        )
        assert(response.status === 200, "Invalid response status");
        return response.data
    }
    async getGames(): Promise<Match[]> {
        const response = await axios.get(`${URL}/matches`,
            {
                method: "GET",
                headers: {
                    Authorization: `Bearer ${this.token}`,
                }
            }
        )
        assert(response.status === 200, "Invalid response status");
        return response.data;
    }
}
async function main() {
    const client1 = new Client();
    const client2 = new Client();

    const match = await client1.createGame();
    console.log({ match });
    const match2 = await client2.joinGame(match.id);
    const match3 = await client1.getGame(match.id);
    console.log({ match, match2, match3 });
    deepEqual(match3, match2, "Matches not equal");
    const matches = await client2.getGames();
    console.log(matches);
}
main().then(() => console.log("DONE"));
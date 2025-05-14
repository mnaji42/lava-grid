"use client"
import { useEffect, useState } from "react"
import { useRouter, useParams } from "next/navigation"

export default function GamePage() {
  const [ws, setWs] = useState<WebSocket | null>(null)
  const [gameState, setGameState] = useState(null)
  const [players, setPlayers] = useState([])
  const [connectionStatus, setConnectionStatus] = useState("Connecting...")
  const { gameId } = useParams()
  const router = useRouter()

  // Gestion du wallet mock
  const [wallet, setWallet] = useState<string | null>(null)
  const [username, setUsername] = useState<string>("")

  useEffect(() => {
    if (typeof window !== "undefined") {
      const w = localStorage.getItem("wallet")
      if (!w) {
        router.replace("/")
        return
      }
      setWallet(w)
      setUsername(localStorage.getItem("username") || "")
    }
  }, [router])

  // Logout = suppression du wallet et du username
  const handleLogout = () => {
    localStorage.removeItem("wallet")
    localStorage.removeItem("username")
    setWallet(null)
    router.replace("/")
  }

  useEffect(() => {
    if (!wallet) return
    const socket = new WebSocket(
      `ws://localhost:8080/ws/game/${gameId}?wallet=${encodeURIComponent(
        wallet
      )}&username=${encodeURIComponent(username)}`
    )

    socket.onopen = () => {
      setConnectionStatus("Connected")
      console.log("Connected to game session")
    }

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data)
      switch (msg.action) {
        case "GameStateUpdate":
          setGameState(msg.data.state)
          setPlayers(msg.data.players || [])
          break
        case "GameEnded":
          router.push(`/game-over/${msg.data.winner}`)
          break
        default:
          console.warn("Unhandled message type:", msg.action)
      }
    }

    socket.onclose = () => {
      setConnectionStatus("Disconnected")
      console.log("Game connection closed")
      // router.push("/")
    }

    setWs(socket)

    return () => {
      socket.close()
    }
  }, [gameId, wallet, username, router])

  const sendMove = (direction: string) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(
        JSON.stringify({
          action: "PlayerMove",
          direction: direction,
        })
      )
    }
  }

  // Gestion des touches clavier
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowUp":
          sendMove("Up")
          break
        case "ArrowDown":
          sendMove("Down")
          break
        case "ArrowLeft":
          sendMove("Left")
          break
        case "ArrowRight":
          sendMove("Right")
          break
        default:
          break
      }
    }

    window.addEventListener("keydown", handleKeyPress)
    return () => window.removeEventListener("keydown", handleKeyPress)
  }, [ws])

  if (!wallet) {
    return <div className="p-4 text-center">Redirecting...</div>
  }

  if (!gameState) {
    return (
      <div className="p-4 text-center">
        <div className="mb-2">
          <span className="text-xs text-slate-500">Your wallet:</span>
          <span className="font-mono bg-slate-100 px-2 py-1 rounded text-slate-800 text-xs ml-2">
            {wallet}
          </span>
          <button
            onClick={handleLogout}
            className="ml-4 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
          >
            Logout
          </button>
        </div>
        {connectionStatus}
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      <div className="max-w-2xl mx-auto">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl">Partie #{gameId}</h1>
          <div>
            <span className="text-xs text-slate-400">Wallet:</span>
            <span className="font-mono bg-slate-800 px-2 py-1 rounded text-xs ml-2">
              {wallet}
            </span>
            <button
              onClick={handleLogout}
              className="ml-4 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
            >
              Logout
            </button>
          </div>
        </div>
        <div className="mb-4">Status: {connectionStatus}</div>

        {/* Grille de jeu */}
        <div
          className="grid gap-1 bg-gray-800 p-2 rounded-lg"
          style={{
            gridTemplateColumns: `repeat(${gameState.grid[0].length}, minmax(0, 1fr))`,
          }}
        >
          {gameState.grid.map((row, y) =>
            row.map((cell, x) => (
              <div
                key={`${x}-${y}`}
                className={`aspect-square flex items-center justify-center
                                    ${
                                      cell === "Solid"
                                        ? "bg-gray-700"
                                        : "bg-red-900"
                                    }
                                    ${
                                      gameState.players &&
                                      gameState.players.some(
                                        (p) =>
                                          p.position.x === x &&
                                          p.position.y === y
                                      )
                                        ? "bg-blue-500"
                                        : ""
                                    }`}
              >
                {cell === "Broken" && "🔥"}
              </div>
            ))
          )}
        </div>

        {/* Contrôles tactiles */}
        <div className="mt-4 grid grid-cols-3 gap-2">
          <button
            onClick={() => sendMove("Up")}
            className="col-start-2 bg-gray-700 p-2"
          >
            ↑
          </button>
          <button onClick={() => sendMove("Left")} className="bg-gray-700 p-2">
            ←
          </button>
          <button onClick={() => sendMove("Stay")} className="bg-gray-700 p-2">
            ⏸
          </button>
          <button onClick={() => sendMove("Right")} className="bg-gray-700 p-2">
            →
          </button>
          <button
            onClick={() => sendMove("Down")}
            className="col-start-2 bg-gray-700 p-2"
          >
            ↓
          </button>
        </div>
      </div>
    </div>
  )
}

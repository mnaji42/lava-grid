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

  // Récupération du pseudo depuis le localStorage
  const [username] = useState(() => {
    if (typeof window !== "undefined") {
      return localStorage.getItem("username") || ""
    }
    return ""
  })

  useEffect(() => {
    const socket = new WebSocket(
      `ws://localhost:8080/ws/game/${gameId}?username=${encodeURIComponent(
        username
      )}`
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
          setPlayers(msg.data.players)
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
  }, [gameId, username, router])

  const sendMove = (direction) => {
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
    const handleKeyPress = (e) => {
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

  if (!gameState) {
    return <div className="p-4 text-center">{connectionStatus}</div>
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      <div className="max-w-2xl mx-auto">
        <h1 className="text-2xl mb-4">Partie #{gameId}</h1>
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

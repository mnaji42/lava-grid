"use client"
import { useEffect, useRef, useState } from "react"
import { useRouter, useParams } from "next/navigation"

type Player = {
  id: number
  username: string
  pos: { x: number; y: number }
  cannonball_count: number
  is_alive: boolean
}

type Cannonball = {
  pos: { x: number; y: number }
}

type GameState = {
  grid: string[][]
  players: Player[]
  cannonballs: Cannonball[]
  turn: number
  targeted_tiles: any[]
}

type GameWsMessage =
  | {
      action: "GameStateUpdate"
      data: { state: GameState; turn_duration: number }
    }
  | { action: "GameEnded"; data: { winner: string } }
  | { action: string; data: any }

export default function GamePage() {
  const [ws, setWs] = useState<WebSocket | null>(null)
  const [gameState, setGameState] = useState<GameState | null>(null)
  const [connectionStatus, setConnectionStatus] = useState("Connecting...")
  const [wallet, setWallet] = useState<string | null>(null)
  const [username, setUsername] = useState<string>("")
  const [mounted, setMounted] = useState(false)

  // Timer state
  const [turnTimer, setTurnTimer] = useState<number>(0)
  const [turnDuration, setTurnDuration] = useState<number>(0)
  const [phase, setPhase] = useState<"move" | "wait" | "idle">("idle")
  const [hasPlayed, setHasPlayed] = useState<boolean>(false)
  const timerRef = useRef<NodeJS.Timeout | null>(null)
  const lastTurnRef = useRef<number>(0)

  // Pour la prévisualisation locale du déplacement du joueur
  const [localPlayerPos, setLocalPlayerPos] = useState<{
    x: number
    y: number
  } | null>(null)
  const [localTurn, setLocalTurn] = useState<number>(-1)

  const { gameId } = useParams()
  const router = useRouter()

  useEffect(() => {
    setMounted(true)
  }, [])

  useEffect(() => {
    if (!mounted) return
    const w = localStorage.getItem("wallet")
    if (!w) {
      router.replace("/")
      return
    }
    setWallet(w)
    setUsername(localStorage.getItem("username") || "")
  }, [mounted, router])

  // Logout
  const handleLogout = () => {
    localStorage.removeItem("wallet")
    localStorage.removeItem("username")
    setWallet(null)
    router.replace("/")
  }

  // Timer logic: precise countdown with setInterval
  const startTurnTimer = (duration: number) => {
    if (timerRef.current) clearInterval(timerRef.current)
    setTurnTimer(duration)
    let timeLeft = duration
    timerRef.current = setInterval(() => {
      timeLeft -= 1
      setTurnTimer(timeLeft)
      if (timeLeft <= 0) {
        if (timerRef.current) clearInterval(timerRef.current)
        timerRef.current = null
      }
    }, 1000)
  }

  // Clean up timer on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) clearInterval(timerRef.current)
    }
  }, [])

  // WebSocket
  useEffect(() => {
    if (!wallet || !gameId || !username) return

    const socket = new WebSocket(
      `ws://localhost:8080/ws/game/${gameId}?wallet=${encodeURIComponent(
        wallet
      )}&username=${encodeURIComponent(username)}`
    )

    socket.onopen = () => {
      setConnectionStatus("Connected")
    }

    socket.onmessage = (event) => {
      const msg: GameWsMessage = JSON.parse(event.data)
      switch (msg.action) {
        case "GameStateUpdate": {
          setGameState(msg.data.state)
          setTurnDuration(msg.data.turn_duration)
          // Si nouveau tour, reset phase et timer
          if (lastTurnRef.current !== msg.data.state.turn) {
            lastTurnRef.current = msg.data.state.turn
            setHasPlayed(false)
            setPhase("move")
            // Timer = TURN_DURATION - 1s (latence safe)
            const safeDuration = msg.data.turn_duration - 1
            startTurnTimer(safeDuration)
            // Reset la position locale au début du tour
            const currentPlayer = msg.data.state.players.find(
              (p) => p.username === username
            )
            setLocalPlayerPos(currentPlayer ? { ...currentPlayer.pos } : null)
            setLocalTurn(msg.data.state.turn)
          }
          break
        }
        case "GameEnded":
          router.push(`/game-over/${msg.data.winner}`)
          break
        default:
          // Optionally handle other actions
          break
      }
    }

    socket.onclose = () => {
      setConnectionStatus("Disconnected")
      if (timerRef.current) clearInterval(timerRef.current)
    }

    setWs(socket)
    return () => {
      socket.close()
      if (timerRef.current) clearInterval(timerRef.current)
    }
  }, [gameId, wallet, username, router])

  // Applique la règle de déplacement locale
  function computeNewPos(
    direction: "Up" | "Down" | "Left" | "Right",
    pos: { x: number; y: number },
    grid: string[][]
  ) {
    let new_pos = { ...pos }
    switch (direction) {
      case "Up":
        if (new_pos.y > 0) new_pos.y -= 1
        break
      case "Down":
        if (new_pos.y < grid.length - 1) new_pos.y += 1
        break
      case "Left":
        if (new_pos.x > 0) new_pos.x -= 1
        break
      case "Right":
        if (new_pos.x < grid[0].length - 1) new_pos.x += 1
        break
    }
    return new_pos
  }

  // Envoi des mouvements + preview local
  const sendMove = (direction: string) => {
    if (
      ws &&
      ws.readyState === WebSocket.OPEN &&
      phase === "move" &&
      !hasPlayed &&
      gameState
    ) {
      // Preview local
      const currentPlayer = gameState.players.find(
        (p) => p.username === username
      )
      if (currentPlayer && localPlayerPos && localTurn === gameState.turn) {
        const newPos = computeNewPos(
          direction as "Up" | "Down" | "Left" | "Right",
          localPlayerPos,
          gameState.grid
        )
        setLocalPlayerPos(newPos)
      }
      ws.send(
        JSON.stringify({
          Move: direction, // format attendu par le backend Rust
        })
      )
      setHasPlayed(true)
      setPhase("wait")
    }
  }

  // Gestion des touches clavier
  useEffect(() => {
    if (!ws) return
    const handleKeyPress = (e: KeyboardEvent) => {
      if (phase !== "move" || hasPlayed || !gameState) return
      let direction: "Up" | "Down" | "Left" | "Right" | null = null
      switch (e.key) {
        case "ArrowUp":
          direction = "Up"
          break
        case "ArrowDown":
          direction = "Down"
          break
        case "ArrowLeft":
          direction = "Left"
          break
        case "ArrowRight":
          direction = "Right"
          break
        default:
          break
      }
      if (direction) {
        e.preventDefault()
        sendMove(direction)
      }
    }
    window.addEventListener("keydown", handleKeyPress)
    return () => window.removeEventListener("keydown", handleKeyPress)
  }, [ws, phase, hasPlayed, gameState, localPlayerPos, localTurn])

  // Sécuriser le rendu : attendre que le composant soit monté et que le wallet soit chargé
  if (!mounted) return null
  if (!wallet) return <div className="p-4 text-center">Redirecting...</div>
  if (!gameState)
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

  // Helper pour afficher les cannonballs
  const renderCannonballs = (count: number) =>
    Array.from({ length: count }).map((_, i) => (
      <span key={i} role="img" aria-label="cannonball" className="ml-0.5">
        💣
      </span>
    ))

  // Helper pour savoir si c'est le joueur courant
  const isCurrentPlayer = (player: Player) => {
    return player.username === username
  }

  // Affichage du timer et du message d'action
  let actionMessage = ""
  if (phase === "move") {
    actionMessage = hasPlayed ? "Waiting for other players..." : "Your turn!"
  } else if (phase === "wait") {
    actionMessage = "Waiting for other players..."
  }

  // Pour la grille, on veut afficher la position locale du joueur courant si elle existe
  function getPlayerPos(player: Player): { x: number; y: number } {
    if (
      isCurrentPlayer(player) &&
      localPlayerPos &&
      localTurn === gameState.turn
    ) {
      return localPlayerPos
    }
    return player.pos
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      <div className="max-w-2xl mx-auto">
        {/* Header d'infos de partie */}
        <div className="flex flex-col md:flex-row md:items-center md:justify-between mb-6 gap-2">
          <div>
            <h1 className="text-2xl font-bold mb-1">Partie #{gameId}</h1>
            <div className="text-xs text-slate-400">
              Status: <span className="font-semibold">{connectionStatus}</span>
            </div>
            <div className="text-xs text-slate-400">
              Tour actuel:{" "}
              <span className="font-semibold text-yellow-300">
                {gameState.turn}
              </span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-slate-400">Wallet:</span>
            <span className="font-mono bg-slate-800 px-2 py-1 rounded text-xs">
              {wallet}
            </span>
            <button
              onClick={handleLogout}
              className="ml-2 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
            >
              Logout
            </button>
          </div>
        </div>

        {/* Liste des joueurs */}
        <div className="mb-4 bg-gray-800 rounded-lg p-3 shadow flex flex-col gap-2">
          <div className="font-semibold text-slate-300 mb-1">Joueurs</div>
          <div className="flex flex-wrap gap-2">
            {gameState.players.map((player) => (
              <div
                key={player.id}
                className={`
                  flex items-center gap-2 px-3 py-2 rounded transition
                  ${
                    player.is_alive
                      ? isCurrentPlayer(player)
                        ? "bg-blue-700/80 border-2 border-blue-400 shadow"
                        : "bg-gray-700/80"
                      : "bg-red-900/80 border border-red-700 opacity-60"
                  }
                `}
              >
                <span
                  className={`
                    font-bold text-base
                    ${player.is_alive ? "" : "line-through text-red-400"}
                    ${
                      isCurrentPlayer(player)
                        ? "underline underline-offset-2"
                        : ""
                    }
                  `}
                  title={isCurrentPlayer(player) ? "C'est vous !" : ""}
                >
                  {player.username || `Joueur ${player.id}`}
                </span>
                <span className="flex items-center ml-2">
                  {renderCannonballs(player.cannonball_count)}
                </span>
                {!player.is_alive && (
                  <span className="ml-2 text-xs text-red-400 font-semibold">
                    💀 Mort
                  </span>
                )}
                {isCurrentPlayer(player) && player.is_alive && (
                  <span className="ml-2 text-xs text-blue-300 font-semibold">
                    👈 Vous
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
        <div className="relative">
          {/* Timer et message d'action */}
          <div className="absolute left-[50%] top-[50%] -translate-x-1/2 -translate-y-1/2 w-full flex items-center justify-center mb-4 gap-4">
            <div className="text-lg font-semibold opacity-50">
              {actionMessage}
            </div>
            {phase !== "idle" && (
              <div className="text-2xl font-bold text-blue-400 bg-gray-800 px-4 py-1 rounded shadow animate-pulse opacity-80">
                {turnTimer}s
              </div>
            )}
          </div>

          {/* Grille de jeu */}
          <div
            className="grid gap-1 bg-gray-800 p-2 rounded-lg"
            style={{
              gridTemplateColumns: `repeat(${gameState.grid[0].length}, minmax(0, 1fr))`,
            }}
          >
            {gameState.grid.map((row, y) =>
              row.map((cell, x) => {
                // Vérifier si un joueur est sur cette case (en tenant compte du local preview)
                const playerHere = gameState.players.some((p) => {
                  const pos = getPlayerPos(p)
                  return pos.x === x && pos.y === y && p.is_alive
                })
                // Vérifier si un boulet de canon est sur cette case
                const cannonballHere = gameState.cannonballs.some(
                  (c) => c.pos.x === x && c.pos.y === y
                )
                return (
                  <div
                    key={`${x}-${y}`}
                    className={`aspect-square flex items-center justify-center
                    ${cell === "Solid" ? "bg-gray-700" : "bg-red-900"}
                    ${playerHere ? "bg-blue-500" : ""}
                    ${cannonballHere ? "border-4 border-yellow-400" : ""}
                    rounded transition
                  `}
                  >
                    {cell === "Broken" && "🔥"}
                    {playerHere && (
                      <span role="img" aria-label="player">
                        🧑
                      </span>
                    )}
                    {cannonballHere && (
                      <span role="img" aria-label="cannonball">
                        💣
                      </span>
                    )}
                  </div>
                )
              })
            )}
          </div>
        </div>

        {/* Contrôles tactiles (optionnel, décommenter si besoin) */}
        {/* <div className="mt-4 grid grid-cols-3 gap-2">
          <button
            onClick={() => sendMove("Up")}
            className="col-start-2 bg-gray-700 p-2"
            disabled={phase !== "move" || hasPlayed}
          >
            ↑
          </button>
          <button onClick={() => sendMove("Left")} className="bg-gray-700 p-2" disabled={phase !== "move" || hasPlayed}>
            ←
          </button>
          <button onClick={() => sendMove("Stay")} className="bg-gray-700 p-2" disabled={phase !== "move" || hasPlayed}>
            ⏸
          </button>
          <button onClick={() => sendMove("Right")} className="bg-gray-700 p-2" disabled={phase !== "move" || hasPlayed}>
            →
          </button>
          <button
            onClick={() => sendMove("Down")}
            className="col-start-2 bg-gray-700 p-2"
            disabled={phase !== "move" || hasPlayed}
          >
            ↓
          </button>
        </div> */}
      </div>
    </div>
  )
}

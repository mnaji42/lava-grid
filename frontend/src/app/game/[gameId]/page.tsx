// "use client"
// import { useEffect, useState } from "react"
// import { useRouter, useParams } from "next/navigation"

// type Player = {
//   id: number
//   pos: { x: number; y: number }
//   cannonball_count: number
//   is_alive: boolean
// }

// type Cannonball = {
//   pos: { x: number; y: number }
// }

// type GameState = {
//   grid: string[][]
//   players: Player[]
//   cannonballs: Cannonball[]
//   turn: number
//   targeted_tiles: any[]
// }

// export default function GamePage() {
//   const [ws, setWs] = useState<WebSocket | null>(null)
//   const [gameState, setGameState] = useState<GameState | null>(null)
//   const [connectionStatus, setConnectionStatus] = useState("Connecting...")
//   const [wallet, setWallet] = useState<string | null>(null)
//   const [username, setUsername] = useState<string>("")
//   const [mounted, setMounted] = useState(false)

//   const { gameId } = useParams()
//   const router = useRouter()

//   // S'assurer que le composant est monté côté client avant d'accéder à localStorage
//   useEffect(() => {
//     setMounted(true)
//   }, [])

//   // Récupérer wallet/username une fois monté
//   useEffect(() => {
//     if (!mounted) return
//     const w = localStorage.getItem("wallet")
//     if (!w) {
//       router.replace("/")
//       return
//     }
//     setWallet(w)
//     setUsername(localStorage.getItem("username") || "")
//   }, [mounted, router])

//   // Logout
//   const handleLogout = () => {
//     localStorage.removeItem("wallet")
//     localStorage.removeItem("username")
//     setWallet(null)
//     router.replace("/")
//   }

//   // WebSocket
//   useEffect(() => {
//     if (!wallet || !gameId || !username) return

//     const socket = new WebSocket(
//       `ws://localhost:8080/ws/game/${gameId}?wallet=${encodeURIComponent(
//         wallet
//       )}&username=${encodeURIComponent(username)}`
//     )

//     socket.onopen = () => {
//       setConnectionStatus("Connected")
//     }

//     socket.onmessage = (event) => {
//       const msg = JSON.parse(event.data)
//       console.log(msg.data.state)
//       switch (msg.action) {
//         case "GameStateUpdate":
//           setGameState(msg.data.state)
//           break
//         case "GameEnded":
//           router.push(`/game-over/${msg.data.winner}`)
//           break
//         default:
//           // Optionally handle other actions
//           break
//       }
//     }

//     socket.onclose = () => {
//       setConnectionStatus("Disconnected")
//     }

//     setWs(socket)
//     return () => {
//       socket.close()
//     }
//   }, [gameId, wallet, username, router])

//   // Envoi des mouvements
//   const sendMove = (direction: string) => {
//     if (ws && ws.readyState === WebSocket.OPEN) {
//       ws.send(
//         JSON.stringify({
//           Move: direction, // <-- format attendu par le backend Rust
//         })
//       )
//     }
//   }

//   // Gestion des touches clavier
//   useEffect(() => {
//     if (!ws) return
//     const handleKeyPress = (e: KeyboardEvent) => {
//       switch (e.key) {
//         case "ArrowUp":
//         case "ArrowDown":
//         case "ArrowLeft":
//         case "ArrowRight":
//           e.preventDefault() // Empêche le scroll avec les flèches
//           sendMove(e.key.replace("Arrow", ""))
//           break
//         default:
//           break
//       }
//     }
//     window.addEventListener("keydown", handleKeyPress)
//     return () => window.removeEventListener("keydown", handleKeyPress)
//   }, [ws])

//   // Sécuriser le rendu : attendre que le composant soit monté et que le wallet soit chargé
//   if (!mounted) return null
//   if (!wallet) return <div className="p-4 text-center">Redirecting...</div>
//   if (!gameState)
//     return (
//       <div className="p-4 text-center">
//         <div className="mb-2">
//           <span className="text-xs text-slate-500">Your wallet:</span>
//           <span className="font-mono bg-slate-100 px-2 py-1 rounded text-slate-800 text-xs ml-2">
//             {wallet}
//           </span>
//           <button
//             onClick={handleLogout}
//             className="ml-4 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
//           >
//             Logout
//           </button>
//         </div>
//         {connectionStatus}
//       </div>
//     )

//   return (
//     <div className="min-h-screen bg-gray-900 text-white p-4">
//       <div className="max-w-2xl mx-auto">
//         <div className="flex items-center justify-between mb-4">
//           <h1 className="text-2xl">Partie #{gameId}</h1>
//           <div>
//             <span className="text-xs text-slate-400">Wallet:</span>
//             <span className="font-mono bg-slate-800 px-2 py-1 rounded text-xs ml-2">
//               {wallet}
//             </span>
//             <button
//               onClick={handleLogout}
//               className="ml-4 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
//             >
//               Logout
//             </button>
//           </div>
//         </div>
//         <div className="mb-4">Status: {connectionStatus}</div>

//         {/* Grille de jeu */}
//         <div
//           className="grid gap-1 bg-gray-800 p-2 rounded-lg"
//           style={{
//             gridTemplateColumns: `repeat(${gameState.grid[0].length}, minmax(0, 1fr))`,
//           }}
//         >
//           {gameState.grid.map((row, y) =>
//             row.map((cell, x) => {
//               // Vérifier si un joueur est sur cette case
//               const playerHere = gameState.players.some(
//                 (p) => p.pos.x === x && p.pos.y === y
//               )
//               // Vérifier si un boulet de canon est sur cette case
//               const cannonballHere = gameState.cannonballs.some(
//                 (c) => c.pos.x === x && c.pos.y === y
//               )
//               return (
//                 <div
//                   key={`${x}-${y}`}
//                   className={`aspect-square flex items-center justify-center
//                     ${cell === "Solid" ? "bg-gray-700" : "bg-red-900"}
//                     ${playerHere ? "bg-blue-500" : ""}
//                     ${cannonballHere ? "border-4 border-yellow-400" : ""}
//                   `}
//                 >
//                   {cell === "Broken" && "🔥"}
//                   {playerHere && (
//                     <span role="img" aria-label="player">
//                       🧑
//                     </span>
//                   )}
//                   {cannonballHere && (
//                     <span role="img" aria-label="cannonball">
//                       💣
//                     </span>
//                   )}
//                 </div>
//               )
//             })
//           )}
//         </div>

//         {/* Contrôles tactiles */}
//         {/* <div className="mt-4 grid grid-cols-3 gap-2">
//           <button
//             onClick={() => sendMove("Up")}
//             className="col-start-2 bg-gray-700 p-2"
//           >
//             ↑
//           </button>
//           <button onClick={() => sendMove("Left")} className="bg-gray-700 p-2">
//             ←
//           </button>
//           <button onClick={() => sendMove("Stay")} className="bg-gray-700 p-2">
//             ⏸
//           </button>
//           <button onClick={() => sendMove("Right")} className="bg-gray-700 p-2">
//             →
//           </button>
//           <button
//             onClick={() => sendMove("Down")}
//             className="col-start-2 bg-gray-700 p-2"
//           >
//             ↓
//           </button>
//         </div> */}
//       </div>
//     </div>
//   )
// }

"use client"
import { useEffect, useState } from "react"
import { useRouter, useParams } from "next/navigation"

type Player = {
  id: number
  username: string // <-- doit être envoyé par le backend !
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

export default function GamePage() {
  const [ws, setWs] = useState<WebSocket | null>(null)
  const [gameState, setGameState] = useState<GameState | null>(null)
  const [connectionStatus, setConnectionStatus] = useState("Connecting...")
  const [wallet, setWallet] = useState<string | null>(null)
  const [username, setUsername] = useState<string>("")
  const [mounted, setMounted] = useState(false)

  const { gameId } = useParams()
  const router = useRouter()

  // S'assurer que le composant est monté côté client avant d'accéder à localStorage
  useEffect(() => {
    setMounted(true)
  }, [])

  // Récupérer wallet/username une fois monté
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
      const msg = JSON.parse(event.data)
      console.log(msg.data.state)
      switch (msg.action) {
        case "GameStateUpdate":
          setGameState(msg.data.state)
          break
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
    }

    setWs(socket)
    return () => {
      socket.close()
    }
  }, [gameId, wallet, username, router])

  // Envoi des mouvements
  const sendMove = (direction: string) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(
        JSON.stringify({
          Move: direction, // <-- format attendu par le backend Rust
        })
      )
    }
  }

  // Gestion des touches clavier
  useEffect(() => {
    if (!ws) return
    const handleKeyPress = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowUp":
        case "ArrowDown":
        case "ArrowLeft":
        case "ArrowRight":
          e.preventDefault() // Empêche le scroll avec les flèches
          sendMove(e.key.replace("Arrow", ""))
          break
        default:
          break
      }
    }
    window.addEventListener("keydown", handleKeyPress)
    return () => window.removeEventListener("keydown", handleKeyPress)
  }, [ws])

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
    // On compare l'id du joueur avec le wallet courant (à adapter si id != wallet)
    // Ici, on suppose que le backend renvoie un champ "id" qui correspond à l'index ou à l'adresse
    // Si c'est un index, il faut adapter la logique
    return player.username === username
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

        {/* Grille de jeu */}
        <div
          className="grid gap-1 bg-gray-800 p-2 rounded-lg"
          style={{
            gridTemplateColumns: `repeat(${gameState.grid[0].length}, minmax(0, 1fr))`,
          }}
        >
          {gameState.grid.map((row, y) =>
            row.map((cell, x) => {
              // Vérifier si un joueur est sur cette case
              const playerHere = gameState.players.some(
                (p) => p.pos.x === x && p.pos.y === y && p.is_alive
              )
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

        {/* Contrôles tactiles (optionnel, décommenter si besoin) */}
        {/* <div className="mt-4 grid grid-cols-3 gap-2">
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
        </div> */}
      </div>
    </div>
  )
}

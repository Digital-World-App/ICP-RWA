import { useState } from 'react';
import { createActor, canisterId } from 'declarations/Digital_World_backend';

function App() {
  const [greeting, setGreeting] = useState('');
  const [result, setResult] = useState('');
  const [digitalWorldId, setDigitalWorldId] = useState('');
  const [amount, setAmount] = useState('');

  const actor = createActor(canisterId);

  function handleGreetSubmit(event) {
    event.preventDefault();
    const name = event.target.elements.name.value;
    actor.greet(name).then((greeting) => {
      setGreeting(greeting);
    });
  }

  async function handleBuyItem(event) {
    event.preventDefault();
    try {
      const buyResult = await actor.buy_item(Number(digitalWorldId), BigInt(amount));
      setResult(`Compra realizada com sucesso: ${buyResult}`);
    } catch (error) {
      setResult(`Erro ao realizar compra: ${error.message}`);
    }
  }

  async function handleClaimSale(event) {
    event.preventDefault();
    try {
      const claimResult = await actor.claim_sale(Number(digitalWorldId));
      setResult(`Reivindicação de venda realizada: ${claimResult}`);
    } catch (error) {
      setResult(`Erro ao reivindicar venda: ${error.message}`);
    }
  }

  return (
    <main>
      <img src="/logo2.svg" alt="DFINITY logo" />
      <br />
      <br />

      {/* Formulário para o Greeting */}
      <form onSubmit={handleGreetSubmit}>
        <label htmlFor="name">Enter your name: &nbsp;</label>
        <input id="name" alt="Name" type="text" />
        <button type="submit">Click Me!</button>
      </form>
      <section id="greeting">{greeting}</section>

      <br />
      <br />

      {/* Formulário para Buy Item */}
      <form onSubmit={handleBuyItem}>
        <label htmlFor="digitalWorldId">Digital World ID: &nbsp;</label>
        <input
          id="digitalWorldId"
          type="text"
          value={digitalWorldId}
          onChange={(e) => setDigitalWorldId(e.target.value)}
        />
        <br />
        <label htmlFor="amount">Amount: &nbsp;</label>
        <input
          id="amount"
          type="text"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
        />
        <button type="submit">Comprar Item</button>
      </form>

      <br />

      {/* Formulário para Claim Sale */}
      <form onSubmit={handleClaimSale}>
        <label htmlFor="digitalWorldIdClaim">Digital World ID: &nbsp;</label>
        <input
          id="digitalWorldIdClaim"
          type="text"
          value={digitalWorldId}
          onChange={(e) => setDigitalWorldId(e.target.value)}
        />
        <button type="submit">Reivindicar Venda</button>
      </form>

      <section id="result">{result}</section>
    </main>
  );
}

export default App;

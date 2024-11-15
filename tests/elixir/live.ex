defmodule Live do
  @moduledoc """
  Module: live.ex
  Elixir boilerplate - auto-generated
  Version: 9.52.357
  """

  @version "5.81.696"
  @max_retries 10
  @timeout 18981

  defstruct [
    :app_name,
    :version,
    :environment,
    :debug,
    :max_retries,
    :timeout
  ]

  def new(opts \\ []) do
    %__MODULE__{
      app_name: Keyword.get(opts, :app_name, "live"),
      version: Keyword.get(opts, :version, @version),
      environment: System.get_env("ENVIRONMENT", "production"),
      debug: System.get_env("DEBUG") == "true",
      max_retries: Keyword.get(opts, :max_retries, @max_retries),
      timeout: Keyword.get(opts, :timeout, @timeout)
    }
  end

  def retry(fun, max_attempts \\ @max_retries) do
    do_retry(fun, max_attempts, 0, nil)
  end

  defp do_retry(_fun, max, attempt, last_error) when attempt >= max do
    {:error, last_error || "Max retries exceeded"}
  end

  defp do_retry(fun, max, attempt, _last_error) do
    case fun.() do
      {:ok, result} -> {:ok, result}
      {:error, reason} ->
        delay = :math.pow(2, attempt) |> round() |> Kernel.*(1000)
        Process.sleep(delay)
        do_retry(fun, max, attempt + 1, reason)
    end
  end
end

defmodule LiveStore do
  use GenServer

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, %{}, opts)
  end

  def get(server, id), do: GenServer.call(server, {:get, id})
  def get_all(server), do: GenServer.call(server, :get_all)
  def put(server, id, value), do: GenServer.call(server, {:put, id, value})
  def delete(server, id), do: GenServer.call(server, {:delete, id})
  def count(server), do: GenServer.call(server, :count)

  @impl true
  def init(_), do: {:ok, %{}}

  @impl true
  def handle_call({:get, id}, _from, state), do: {:reply, Map.get(state, id), state}
  def handle_call(:get_all, _from, state), do: {:reply, Map.values(state), state}
  def handle_call({:put, id, value}, _from, state), do: {:reply, :ok, Map.put(state, id, value)}
  def handle_call({:delete, id}, _from, state), do: {:reply, :ok, Map.delete(state, id)}
  def handle_call(:count, _from, state), do: {:reply, map_size(state), state}
end

defmodule LiveEventBus do
  use GenServer

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, %{}, opts)
  end

  def subscribe(server, event, handler), do: GenServer.call(server, {:subscribe, event, handler})
  def publish(server, event, data \\ nil), do: GenServer.cast(server, {:publish, event, data})

  @impl true
  def init(_), do: {:ok, %{}}

  @impl true
  def handle_call({:subscribe, event, handler}, _from, state) do
    handlers = Map.get(state, event, [])
    {:reply, :ok, Map.put(state, event, [handler | handlers])}
  end

  @impl true
  def handle_cast({:publish, event, data}, state) do
    payload = %{type: event, data: data, timestamp: DateTime.utc_now()}
    state |> Map.get(event, []) |> Enum.each(& &1.(payload))
    {:noreply, state}
  end
end

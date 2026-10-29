import {
  useCallback,
  useEffect,
  useEffectEvent,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import {
  useQueryClient,
  type QueryKey,
  type UseQueryResult,
} from "@tanstack/react-query";
import {
  referenceQueryMeta,
  retireReferenceQueries,
} from "../queries/referenceQueries";

type GenerationState = {
  generation: string | null;
  epoch: number;
  recovering: boolean;
};

type RequestTicket = {
  token: string;
  epoch: number;
  lifetime: number;
  signal?: AbortSignal;
};

export function useReferenceGeneration(token: string, active: boolean) {
  const queryClient = useQueryClient();
  const [state, setState] = useState<GenerationState>({
    generation: null,
    epoch: 0,
    recovering: false,
  });
  const live = useRef({ ...state, token, active });
  const lifetime = useRef(0);
  const replacement = useRef(0);
  useLayoutEffect(() => {
    live.current = { ...state, token, active: active && !state.recovering };
  }, [state, token, active]);
  useLayoutEffect(() => {
    lifetime.current += 1;
    return () => {
      lifetime.current += 1;
      live.current.active = false;
    };
  }, [token, active, state.epoch, state.recovering]);

  const publish = useCallback((next: GenerationState) => {
    live.current = { ...live.current, ...next };
    setState(next);
  }, []);
  function capture(signal?: AbortSignal): RequestTicket {
    return { token, epoch: state.epoch, lifetime: lifetime.current, signal };
  }
  function current(ticket: RequestTicket) {
    return !ticket.signal?.aborted &&
      live.current.active &&
      !live.current.recovering &&
      ticket.lifetime === lifetime.current &&
      ticket.epoch === live.current.epoch &&
      ticket.token === live.current.token
      ? live.current
      : null;
  }
  function adopt<T>(
    ticket: RequestTicket,
    generation: string,
    seed?: { queryKey: QueryKey; data: T },
  ) {
    const owner = current(ticket);
    if (!owner || owner.generation !== null) return;
    if (seed) {
      queryClient.getQueryCache().build(queryClient, {
        queryKey: seed.queryKey,
        meta: referenceQueryMeta,
      });
      queryClient.setQueryData(seed.queryKey, seed.data);
    }
    publish({ generation, epoch: owner.epoch, recovering: false });
  }
  async function replace(
    generation: string | null,
    prepare?: () => void | Promise<void>,
  ) {
    const operation = ++replacement.current;
    publish({
      generation: live.current.generation,
      epoch: live.current.epoch,
      recovering: true,
    });
    live.current.active = false;
    try {
      await prepare?.();
      if (operation !== replacement.current) return;
      await retireReferenceQueries(queryClient);
      if (operation !== replacement.current) return;
      publish({ generation, epoch: live.current.epoch + 1, recovering: false });
    } catch (error) {
      if (operation === replacement.current)
        publish({
          generation: live.current.generation,
          epoch: live.current.epoch,
          recovering: false,
        });
      throw error;
    }
  }
  function refresh(prepare?: () => void | Promise<void>) {
    if (live.current.recovering) return Promise.resolve();
    return replace(null, prepare);
  }
  async function applyGeneration(
    generation: string | undefined,
    apply: () => void | Promise<void>,
  ) {
    if (generation !== undefined && generation !== live.current.generation)
      await replace(generation, apply);
    else await apply();
  }
  const reset = useCallback(() => {
    replacement.current += 1;
    lifetime.current += 1;
    publish({ generation: null, epoch: live.current.epoch + 1, recovering: false });
  }, [publish]);
  return { ...state, active, capture, current, adopt, refresh, applyGeneration, reset };
}

export function useReferenceGenerationAdoption<T>(
  owner: ReturnType<typeof useReferenceGeneration>,
  query: Pick<
    UseQueryResult<T>,
    "data" | "isSuccess" | "isFetching" | "isPlaceholderData"
  >,
  generation: string | undefined,
  queryKey: QueryKey,
) {
  const adopt = useEffectEvent(() => {
    if (
      !query.isSuccess ||
      query.isFetching ||
      query.isPlaceholderData ||
      query.data === undefined ||
      generation === undefined
    )
      return;
    owner.adopt(owner.capture(), generation, { queryKey, data: query.data });
  });
  useEffect(() => {
    adopt();
  }, [
    owner.active,
    owner.epoch,
    owner.generation,
    owner.recovering,
    query.data,
    query.isSuccess,
    query.isFetching,
    query.isPlaceholderData,
    generation,
  ]);
}

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRef } from 'react';
import { planComplete } from '@/bridge/plan';
import { refreshPlanViews } from './query';

export function useCompletePlan(onDone: () => void) {
  const queryClient = useQueryClient();
  const onDoneRef = useRef(onDone);
  onDoneRef.current = onDone;
  const mutation = useMutation({
    mutationFn: (input: { id: string; result: string }) => planComplete(input.id, input.result),
    onSuccess: async () => {
      await refreshPlanViews(queryClient);
      onDoneRef.current();
    }
  });

  return {
    pending: mutation.isPending,
    error: mutation.error,
    submit: (id: string, result: string) => {
      mutation.mutate({ id, result });
    }
  };
}

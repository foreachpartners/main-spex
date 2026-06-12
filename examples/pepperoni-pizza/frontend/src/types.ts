export interface Order {
  id: number;
  customer: string;
  delivered: boolean;
}

export interface NewOrder {
  customer: string;
  delivered?: boolean;
}

export interface UpdateOrder {
  customer?: string;
  delivered?: boolean;
}

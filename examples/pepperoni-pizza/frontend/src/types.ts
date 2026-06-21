export interface Pizza {
  id: number;
  name: string;
  price: number;
}

export interface Delivery {
  id: number;
  items: string[];
  estimatedSeconds: number;
  remainingSeconds: number;
}

export interface DeliveryEstimate {
  estimatedSeconds: number;
}

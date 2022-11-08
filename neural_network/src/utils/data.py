import os
import torch
import json
import hydra
import pytorch_lightning as pl
from omegaconf import DictConfig
from torch.utils.data import DataLoader, TensorDataset
from tqdm.auto import tqdm

DATA_DIR = "games"

def parse_game_data_from_json():
    policies = []
    values = []
    states = []

    filenames = [os.path.join(DATA_DIR, f) for f in os.listdir(DATA_DIR)]
    for filename in tqdm(filenames):
        with open(filename) as f:
            data = json.load(f)

        for game_data in data:
            states.append(game_data["state"])
            policies.append(game_data["policy"])
            values.append(game_data["value"])

    states = torch.FloatTensor(states)
    policies = torch.FloatTensor(policies)
    values = torch.FloatTensor(values).unsqueeze(1)

    torch.save((states, policies, values), "flat_data.pt")

    print(states.shape)
    print(policies.shape)
    print(values.shape)

    return states, policies, values


class DataModule(pl.LightningDataModule):
    def __init__(self, batch_size: int, shuffle: bool = True, num_workers: int = 1):
        super().__init__()

        self.batch_size = batch_size
        self.shuffle = shuffle
        self.num_workers = num_workers

    def setup(self, stage=None):
        if os.path.exists("flat_data.pt"):
            states, policies, values = torch.load("flat_data.pt")
        else:
            states, policies, values = parse_game_data_from_json()

        self.dataset = TensorDataset(states, policies, values)

    def train_dataloader(self):
        return DataLoader(
            self.dataset, batch_size=self.batch_size, shuffle=self.shuffle, num_workers=self.num_workers
        )


if __name__ == "__main__":
    parse_game_data_from_json()

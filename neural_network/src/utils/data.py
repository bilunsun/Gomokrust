import os
import torch
import json
import hydra
import pytorch_lightning as pl
from omegaconf import DictConfig
from torch.utils.data import DataLoader, TensorDataset, random_split
from tqdm.auto import tqdm

DATA_DIR = "games"

def augment(states, policies, values):
    N = states.size(0)

    states_rotations = [states]
    policies_rotations = [policies]

    turns = states[:, -1].unsqueeze(1)
    states = states[:, :-1].reshape(N, 8, 8)
    policies = policies.reshape(N, 8, 8)

    for k in (1, 2, 3):
        states_rot = torch.rot90(states, k, dims=[1, 2]).reshape(N, -1)
        states_rot = torch.cat((states_rot, turns), dim=-1)
        states_rotations.append(states_rot)
    states = torch.cat(states_rotations)

    for k in (1, 2, 3):
        policies_rot = torch.rot90(policies, k, dims=[1, 2]).reshape(N, -1)
        policies_rotations.append(policies_rot)
    policies = torch.cat(policies_rotations).reshape(N*4, -1)

    values = torch.cat([values] * 4)

    return states, policies, values

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

    # Data augmentation
    states, policies, values = augment(states, policies, values)

    torch.save((states, policies, values), "data_conv.pt")

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
        # if os.path.exists("data.pt"):
        #     states, policies, values = torch.load("data.pt")
        # else:
        states, policies, values = parse_game_data_from_json()

        dataset = TensorDataset(states, policies, values)

        val_len = min(1024, int(len(dataset) * 0.9))
        train_len = len(dataset) - val_len
        self.train_set, self.val_set = random_split(dataset, [train_len, val_len])

    def train_dataloader(self):
        return DataLoader(
            self.train_set, batch_size=self.batch_size, shuffle=self.shuffle, num_workers=self.num_workers
        )

    def val_dataloader(self):
        return DataLoader(
            self.val_set, batch_size=self.batch_size, shuffle=False, num_workers=self.num_workers
        )

if __name__ == "__main__":
    parse_game_data_from_json()
